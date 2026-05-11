import { GroupView } from "./types";

export function formatGroupHeading(group: GroupView): string {
  if (group.heading) {
    return `[${group.heading}]`;
  }

  return `${group.kind}:`;
}

export function formatFileLabel(path: string): string {
  const filename = path.split("/").pop() ?? path;
  const withoutExtension = filename.replace(/\.mlg$/i, "");
  const normalized = withoutExtension.replace(/_/g, " ");

  return normalized
    .split(/\s+/)
    .filter((part) => part.length > 0)
    .map((part) => part[0].toUpperCase() + part.slice(1))
    .join(" ");
}

export function makeFileAnchor(fileIndex: number): string {
  return `file-${fileIndex}`;
}

export function makeGroupAnchor(fileIndex: number, groupIndex: number): string {
  return `group-${fileIndex}-${groupIndex}`;
}
