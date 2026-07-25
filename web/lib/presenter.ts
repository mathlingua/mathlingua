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

/** Returns the directory that contains a source file path. */
export function fileDirectory(path: string): string {
  const segments = contentRelativePath(path).split("/").filter(Boolean);
  segments.pop();
  return segments.join("/");
}

/**
 * One stop in the linear reading order. The collection root and every directory
 * become a `divider` (a centered title page); each source file is a `file`. The
 * sequence is a depth-first, toc-ordered walk, so Next/Prev read like a book.
 */
export type ReaderNode =
  | {
      kind: "divider";
      /** Directory this divider announces; "" is the collection root. */
      directory: string;
      /** Centered title: the collection name, or the directory label. */
      title: string;
    }
  | {
      kind: "file";
      /** Index of this file in the serialized collection view. */
      fileIndex: number;
      /** File path relative to the content root. */
      path: string;
      /** Directory that contains the file. */
      directory: string;
      /** File label shown in navigation. */
      title: string;
    };

/**
 * Builds the linear reading order: a root divider, then each file in toc order,
 * emitting a divider the first time a directory is entered. Assumes `files` is
 * in depth-first order (each directory's subtree contiguous), which the manifest
 * guarantees.
 */
export function buildReaderNodes(
  files: { path: string; title?: string | null }[],
  directories: DirectoryView[],
  collectionTitle: string,
): ReaderNode[] {
  const nodes: ReaderNode[] = [
    { kind: "divider", directory: "", title: collectionTitle },
  ];
  const emitted = new Set<string>([""]);

  files.forEach((file, fileIndex) => {
    const directory = fileDirectory(file.path);

    if (directory) {
      let prefix = "";
      for (const segment of directory.split("/").filter(Boolean)) {
        prefix = prefix ? `${prefix}/${segment}` : segment;
        if (!emitted.has(prefix)) {
          emitted.add(prefix);
          nodes.push({
            kind: "divider",
            directory: prefix,
            title: formatDirectoryLabel(directories, prefix),
          });
        }
      }
    }

    nodes.push({
      kind: "file",
      fileIndex,
      path: file.path,
      directory,
      title: file.title ?? formatFileLabel(file.path),
    });
  });

  return nodes;
}

/** Browser route for a reader node: a directory route for dividers, else a file route. */
export function readerNodeRoute(node: ReaderNode): string {
  return node.kind === "divider"
    ? directoryRoute(node.directory)
    : fileRoute(node.path);
}

/** Index of the divider node for a directory ("" is the root), or -1. */
export function nodeIndexForDirectory(
  nodes: ReaderNode[],
  directory: string,
): number {
  return nodes.findIndex(
    (node) => node.kind === "divider" && node.directory === directory,
  );
}

/** Index of the file node with the given collection file index, or -1. */
export function nodeIndexForFile(
  nodes: ReaderNode[],
  fileIndex: number,
): number {
  return nodes.findIndex(
    (node) => node.kind === "file" && node.fileIndex === fileIndex,
  );
}

/** Resolves a browser pathname to a reader-node index (root divider by default). */
export function resolveReaderNodeIndex(
  pathname: string,
  nodes: ReaderNode[],
): number {
  const routePath = routePathFromPathname(pathname);
  if (!routePath) {
    return Math.max(nodeIndexForDirectory(nodes, ""), 0);
  }

  const fileNodeIndex = nodes.findIndex(
    (node) => node.kind === "file" && fileRoutePath(node.path) === routePath,
  );
  if (fileNodeIndex >= 0) {
    return fileNodeIndex;
  }

  const dividerNodeIndex = nodes.findIndex(
    (node) =>
      node.kind === "divider" &&
      directoryRoutePath(node.directory) === routePath,
  );
  if (dividerNodeIndex >= 0) {
    return dividerNodeIndex;
  }

  return Math.max(nodeIndexForDirectory(nodes, ""), 0);
}

/** One crumb in the content-page breadcrumb trail. */
export interface BreadcrumbCrumb {
  /** Display label for the crumb. */
  label: string;
  /** Directory to navigate to, or `null` for the current (non-clickable) page. */
  directory: string | null;
}

/**
 * Breadcrumb trail for a reader node. The root crumb (directory "") leads to the
 * collection title page; ancestor directory crumbs lead to their divider pages;
 * the final crumb is the current location and is inert.
 */
export function buildNodeBreadcrumb(
  directories: DirectoryView[],
  node: ReaderNode,
  rootLabel: string,
): BreadcrumbCrumb[] {
  if (node.kind === "divider" && node.directory === "") {
    return [{ label: rootLabel, directory: null }];
  }

  const crumbs: BreadcrumbCrumb[] = [{ label: rootLabel, directory: "" }];
  const prefixes: string[] = [];
  if (node.directory) {
    let prefix = "";
    for (const segment of node.directory.split("/").filter(Boolean)) {
      prefix = prefix ? `${prefix}/${segment}` : segment;
      prefixes.push(prefix);
    }
  }

  if (node.kind === "file") {
    for (const prefix of prefixes) {
      crumbs.push({
        label: formatDirectoryLabel(directories, prefix),
        directory: prefix,
      });
    }
    crumbs.push({ label: node.title, directory: null });
  } else {
    prefixes.forEach((prefix, index) => {
      const isSelf = index === prefixes.length - 1;
      crumbs.push({
        label: formatDirectoryLabel(directories, prefix),
        directory: isSelf ? null : prefix,
      });
    });
  }

  return crumbs;
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
