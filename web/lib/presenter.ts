import { GroupView } from "./types";

export type FileBrowserEntry =
  | {
      kind: "directory";
      path: string;
      label: string;
    }
  | {
      kind: "file";
      path: string;
      label: string;
      fileIndex: number;
    };

export function formatGroupHeading(group: GroupView): string {
  if (group.heading) {
    return `[${group.heading}]`;
  }

  return `${group.kind}:`;
}

export function formatFileLabel(path: string): string {
  const filename = normalizePath(path).split("/").pop() ?? path;
  return formatPathSegment(filename.replace(/\.mlg$/i, ""));
}

export function formatPathSegment(segment: string): string {
  const normalized = segment.replace(/_/g, " ");

  return normalized
    .split(/\s+/)
    .filter((part) => part.length > 0)
    .map((part) => part[0].toUpperCase() + part.slice(1))
    .join(" ");
}

export function buildFileBrowserEntries(
  files: { path: string }[],
  directory: string,
): FileBrowserEntry[] {
  const normalizedDirectory = normalizeDirectory(directory);
  const directories = new Map<string, FileBrowserEntry>();
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
        label: formatFileLabel(path),
        fileIndex,
      });
      return;
    }

    const segment = remaining[0];
    const directoryPath = [...directorySegments, segment].join("/");
    if (!directories.has(directoryPath)) {
      directories.set(directoryPath, {
        kind: "directory",
        path: directoryPath,
        label: formatPathSegment(segment),
      });
    }
  });

  return [...entries, ...directories.values()].sort((left, right) =>
    left.label.localeCompare(right.label, undefined, { sensitivity: "base" }),
  );
}

export function parentDirectory(directory: string): string {
  const normalized = normalizeDirectory(directory);
  const segments = normalized.split("/").filter(Boolean);
  segments.pop();
  return segments.join("/");
}

export function firstFileIndexInDirectory(
  files: { path: string }[],
  directory: string,
): number | null {
  const firstFile = buildFileBrowserEntries(files, directory).find(
    (entry) => entry.kind === "file",
  );

  return firstFile?.kind === "file" ? firstFile.fileIndex : null;
}

export function fileDirectory(path: string): string {
  const segments = contentRelativePath(path).split("/").filter(Boolean);
  segments.pop();
  return segments.join("/");
}

export function makeFileAnchor(path: string): string {
  const withoutExtension = contentRelativePath(path).replace(/\.mlg$/i, "");
  const withUnderscores = withoutExtension
    .trim()
    .replace(/\s+/g, "_")
    .replace(/\/+/g, "/");

  return encodeURIComponent(withUnderscores || "untitled");
}

export function makeGroupAnchor(fileIndex: number, groupIndex: number): string {
  return `group-${fileIndex}-${groupIndex}`;
}

function normalizePath(path: string): string {
  return path.replace(/\\/g, "/").replace(/^\/+/, "");
}

function normalizeDirectory(directory: string): string {
  return normalizePath(directory).replace(/\/+$/, "");
}

function contentRelativePath(path: string): string {
  const normalized = normalizePath(path);
  return normalized.startsWith("content/")
    ? normalized.slice("content/".length)
    : normalized;
}

function isInsideDirectory(
  pathSegments: string[],
  directorySegments: string[],
): boolean {
  return directorySegments.every(
    (segment, index) => pathSegments[index] === segment,
  );
}
