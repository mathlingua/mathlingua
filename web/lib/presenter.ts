import { DirectoryView, GroupView } from "./types";

/** One row in the outline browser, either a child directory or source file. */
export type FileBrowserEntry =
  | {
      /** Directory entries drill into a deeper content folder. */
      kind: "directory";
      /** Directory path relative to the content root. */
      path: string;
      /** Human-readable directory label. */
      label: string;
    }
  | {
      /** File entries select a rendered MathLingua source file. */
      kind: "file";
      /** File path relative to the content root. */
      path: string;
      /** Human-readable file label. */
      label: string;
      /** Index of the matching file in the serialized collection view. */
      fileIndex: number;
    };

/** Formats a top-level group label for places that display raw headings. */
export function formatGroupHeading(group: GroupView): string {
  if (group.heading) {
    return `[${group.heading}]`;
  }

  return `${group.kind}:`;
}

/** Converts a source path into a title-cased outline label. */
export function formatFileLabel(path: string): string {
  const filename = normalizePath(path).split("/").pop() ?? path;
  return formatPathSegment(filename.replace(/\.mlg$/i, ""));
}

/** Converts one route or path segment into title-cased display text. */
export function formatPathSegment(segment: string): string {
  const normalized = segment.replace(/_/g, " ");

  return normalized
    .split(/\s+/)
    .filter((part) => part.length > 0)
    .map((part) => part[0].toUpperCase() + part.slice(1))
    .join(" ");
}

/**
 * Builds the entries shown in the current outline directory.
 *
 * Only direct child files and child directories are returned; deeper files stay
 * hidden until the user navigates into their directory.
 */
export function buildFileBrowserEntries(
  files: { path: string; title?: string | null }[],
  directories: DirectoryView[],
  directory: string,
): FileBrowserEntry[] {
  const normalizedDirectory = normalizeDirectory(directory);
  const directoryLabels = directoryLabelMap(directories);
  const directoryEntries = new Map<string, FileBrowserEntry>();
  const entries: FileBrowserEntry[] = [];

  files.forEach((file, fileIndex) => {
    const path = contentRelativePath(file.path);
    const segments = path.split("/").filter(Boolean);
    const directorySegments = normalizedDirectory
      ? normalizedDirectory.split("/")
      : [];

    if (!isInsideDirectory(segments, directorySegments)) {
      return;
    }

    const remaining = segments.slice(directorySegments.length);
    if (remaining.length === 0) {
      return;
    }

    if (remaining.length === 1) {
      entries.push({
        kind: "file",
        path,
        label: file.title ?? formatFileLabel(path),
        fileIndex,
      });
      return;
    }

    const segment = remaining[0];
    const directoryPath = [...directorySegments, segment].join("/");
    if (!directoryEntries.has(directoryPath)) {
      const entry: FileBrowserEntry = {
        kind: "directory",
        path: directoryPath,
        label: directoryLabels.get(directoryPath) ?? formatPathSegment(segment),
      };
      directoryEntries.set(directoryPath, entry);
      entries.push(entry);
    }
  });

  return entries;
}

/** Returns the toc-aware display label for a source directory path. */
export function formatDirectoryLabel(
  directories: DirectoryView[],
  directory: string,
): string {
  const normalizedDirectory = normalizeDirectory(directory);
  const segment =
    normalizedDirectory.split("/").filter(Boolean).at(-1) ??
    normalizedDirectory;

  return (
    directoryLabelMap(directories).get(normalizedDirectory) ??
    formatPathSegment(segment)
  );
}

/** Returns the parent directory path for outline back-navigation. */
export function parentDirectory(directory: string): string {
  const normalized = normalizeDirectory(directory);
  const segments = normalized.split("/").filter(Boolean);
  segments.pop();
  return segments.join("/");
}

/** Returns the first selectable file inside a directory, if one exists. */
export function firstFileIndexInDirectory(
  files: { path: string; title?: string | null }[],
  directories: DirectoryView[],
  directory: string,
): number | null {
  const firstFile = buildFileBrowserEntries(files, directories, directory).find(
    (entry) => entry.kind === "file",
  );

  return firstFile?.kind === "file" ? firstFile.fileIndex : null;
}

/** Returns the directory that contains a source file path. */
export function fileDirectory(path: string): string {
  const segments = contentRelativePath(path).split("/").filter(Boolean);
  segments.pop();
  return segments.join("/");
}

/** Builds the DOM anchor id for a rendered file. */
export function makeFileAnchor(path: string): string {
  return encodeRoutePath(fileRoutePath(path));
}

/** Builds the browser URL path for a source file. */
export function fileRoute(path: string): string {
  const routePath = fileRoutePath(path);
  return routePath ? `/${encodeRoutePath(routePath)}` : "/";
}

/** Converts a source file path into its extensionless route path. */
export function fileRoutePath(path: string): string {
  const withoutExtension = contentRelativePath(path).replace(/\.mlg$/i, "");

  return normalizeRoutePath(withoutExtension);
}

/** Decodes and normalizes the current browser pathname into a route path. */
export function routePathFromPathname(pathname: string): string {
  return normalizeRoutePath(decodeRoutePath(pathname));
}

/** Builds the browser URL path for an outline directory. */
export function directoryRoute(directory: string): string {
  const routePath = directoryRoutePath(directory);
  return routePath ? `/${encodeRoutePath(routePath)}` : "/";
}

/** Converts an outline directory into its normalized route path. */
export function directoryRoutePath(directory: string): string {
  return normalizeRoutePath(directory);
}

/** Normalizes viewer route paths into slash-separated URL fragments. */
function normalizeRoutePath(path: string): string {
  return path
    .trim()
    .replace(/\s+/g, "_")
    .replace(/\/+/g, "/")
    .replace(/^\/+|\/+$/g, "");
}

/** Builds the stable DOM anchor id for one group from its source UUID. */
export function makeGroupAnchor(
  group: Pick<GroupView, "id">,
  fallbackKey: string,
): string {
  return `group-${sanitizeAnchorSegment(group.id || fallbackKey)}`;
}

function sanitizeAnchorSegment(value: string): string {
  return value.replace(/[^A-Za-z0-9_-]/g, "-");
}

/** Normalizes platform-specific file separators into content-style paths. */
function normalizePath(path: string): string {
  return path.replace(/\\/g, "/").replace(/^\/+/, "");
}

/** Normalizes a directory path while preserving the empty root directory. */
function normalizeDirectory(directory: string): string {
  return normalizePath(directory).replace(/\/+$/, "");
}

/** Strips the conventional `content/` prefix from a source path. */
function contentRelativePath(path: string): string {
  const normalized = normalizePath(path);
  return normalized.startsWith("content/")
    ? normalized.slice("content/".length)
    : normalized;
}

function directoryLabelMap(directories: DirectoryView[]): Map<string, string> {
  return new Map(
    directories.map((entry) => {
      const path = contentRelativePath(entry.path);
      const segment = path.split("/").filter(Boolean).at(-1) ?? path;

      return [path, entry.title ?? formatPathSegment(segment)] as const;
    }),
  );
}

/** Percent-encodes each route segment without encoding path separators. */
function encodeRoutePath(path: string): string {
  return path.split("/").map(encodeURIComponent).join("/");
}

/** Decodes each route segment without treating slashes as encoded content. */
function decodeRoutePath(path: string): string {
  return path
    .split("/")
    .map((segment) => decodeURIComponent(segment))
    .join("/");
}

/** Returns true when a path is inside the outline directory being viewed. */
function isInsideDirectory(
  pathSegments: string[],
  directorySegments: string[],
): boolean {
  return directorySegments.every(
    (segment, index) => pathSegments[index] === segment,
  );
}
