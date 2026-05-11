import { ArgumentView, FileView, GroupView } from "./types";

const TITLE_WORD_BREAK = /[._/-]+/g;

export function formatFileTitle(file: FileView): string {
  const fromTitleSection = firstInlineArgument(file, "Title");
  if (fromTitleSection) {
    return cleanQuotedText(fromTitleSection);
  }

  const baseName = file.path.split("/").pop() ?? file.path;
  const withoutExtension = baseName.replace(/\.[^.]+$/, "");
  return toTitleCase(withoutExtension.replace(TITLE_WORD_BREAK, " "));
}

export function formatFileSummary(file: FileView): string {
  const overview = firstTextArgument(file, "overview");
  if (overview) {
    return cleanQuotedText(overview);
  }

  const written = firstTextArgument(file, "written");
  if (written && /\s/.test(cleanQuotedText(written))) {
    return cleanQuotedText(written);
  }

  return `A rendered view of the ${formatFileTitle(
    file,
  ).toLowerCase()} material in this Mathlingua collection.`;
}

export function formatGroupTitle(group: GroupView): string {
  const described = firstInlineArgumentFromGroup(group, group.kind);
  if (described) {
    return stripQuotes(described);
  }

  if (group.heading) {
    return toTitleCase(group.heading.replace(/^[\\/$]+/, "").replace(TITLE_WORD_BREAK, " "));
  }

  return group.kind;
}

export function makeGroupAnchor(fileIndex: number, groupIndex: number): string {
  return `group-${fileIndex}-${groupIndex}`;
}

function firstInlineArgument(file: FileView, label: string): string | null {
  for (const group of file.items) {
    const value = firstInlineArgumentFromGroup(group, label);
    if (value) {
      return value;
    }
  }

  return null;
}

function firstInlineArgumentFromGroup(
  group: GroupView,
  label: string,
): string | null {
  const section = group.sections.find((item) => item.label === label);
  return section?.inline_argument ?? null;
}

function firstTextArgument(file: FileView, label: string): string | null {
  for (const group of file.items) {
    for (const section of group.sections) {
      if (section.label !== label) {
        continue;
      }

      const text = firstTextArgumentFromArguments(section.arguments);
      if (text) {
        return text;
      }
    }
  }

  return null;
}

function firstTextArgumentFromArguments(arguments_: ArgumentView[]): string | null {
  for (const argument of arguments_) {
    if (argument.kind === "text") {
      return argument.text;
    }

    if (argument.kind === "group") {
      const text = firstTextArgumentFromArguments(
        argument.sections.flatMap((section) => section.arguments),
      );
      if (text) {
        return text;
      }
    }
  }

  return null;
}

function cleanQuotedText(text: string): string {
  return stripQuotes(text).trim();
}

function stripQuotes(text: string): string {
  return text.replace(/^"+|"+$/g, "");
}

function toTitleCase(text: string): string {
  return text
    .split(/\s+/)
    .filter((part) => part.length > 0)
    .map((part) => part[0].toUpperCase() + part.slice(1))
    .join(" ");
}
