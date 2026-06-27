"use client";

import katex from "katex";
import type { ReactNode } from "react";
import styles from "./markdown-text.module.css";

/** Props for rendering Markdown prose with inline and display LaTeX. */
interface MarkdownTextProps {
  /** Markdown source after Mathlingua quote stripping. */
  text: string;
}

/** Renders document prose Markdown with KaTeX math support. */
export function MarkdownText({ text }: MarkdownTextProps) {
  return (
    <div className={styles.markdown}>
      {parseBlocks(text).map((block, index) => (
        <MarkdownBlock block={block} key={`${block.kind}-${index}`} />
      ))}
    </div>
  );
}

/** Renders Markdown inline content without adding block wrappers. */
export function MarkdownInline({ text }: MarkdownTextProps) {
  return <>{parseInline(text)}</>;
}

type MarkdownBlock =
  | {
      kind: "paragraph";
      text: string;
    }
  | {
      kind: "heading";
      level: number;
      text: string;
    }
  | {
      kind: "unordered_list" | "ordered_list";
      items: string[];
    }
  | {
      kind: "code";
      text: string;
    }
  | {
      kind: "math";
      latex: string;
    };

function MarkdownBlock({ block }: { block: MarkdownBlock }) {
  switch (block.kind) {
    case "paragraph":
      return <p>{parseInline(block.text)}</p>;
    case "heading":
      return <MarkdownHeading level={block.level} text={block.text} />;
    case "unordered_list":
      return (
        <ul>
          {block.items.map((item, index) => (
            <li key={index}>{parseInline(item)}</li>
          ))}
        </ul>
      );
    case "ordered_list":
      return (
        <ol>
          {block.items.map((item, index) => (
            <li key={index}>{parseInline(item)}</li>
          ))}
        </ol>
      );
    case "code":
      return (
        <pre>
          <code>{block.text}</code>
        </pre>
      );
    case "math":
      return <KatexMath displayMode latex={block.latex} />;
  }
}

function MarkdownHeading({ level, text }: { level: number; text: string }) {
  switch (Math.min(Math.max(level, 1), 6)) {
    case 1:
      return <h1>{parseInline(text)}</h1>;
    case 2:
      return <h2>{parseInline(text)}</h2>;
    case 3:
      return <h3>{parseInline(text)}</h3>;
    case 4:
      return <h4>{parseInline(text)}</h4>;
    case 5:
      return <h5>{parseInline(text)}</h5>;
    default:
      return <h6>{parseInline(text)}</h6>;
  }
}

function parseBlocks(text: string): MarkdownBlock[] {
  const lines = normalizeNewlines(text).split("\n");
  const blocks: MarkdownBlock[] = [];
  let index = 0;

  while (index < lines.length) {
    if (lines[index].trim() === "") {
      index += 1;
      continue;
    }

    const trimmed = lines[index].trim();
    const heading = /^(#{1,6})\s+(.+)$/.exec(trimmed);
    if (heading) {
      blocks.push({
        kind: "heading",
        level: heading[1].length,
        text: heading[2],
      });
      index += 1;
      continue;
    }

    if (trimmed === "$$") {
      const mathLines: string[] = [];
      index += 1;
      while (index < lines.length && lines[index].trim() !== "$$") {
        mathLines.push(lines[index]);
        index += 1;
      }
      if (index < lines.length) {
        index += 1;
      }
      blocks.push({ kind: "math", latex: mathLines.join("\n").trim() });
      continue;
    }

    if (trimmed.startsWith("```")) {
      const codeLines: string[] = [];
      index += 1;
      while (index < lines.length && !lines[index].trim().startsWith("```")) {
        codeLines.push(lines[index]);
        index += 1;
      }
      if (index < lines.length) {
        index += 1;
      }
      blocks.push({ kind: "code", text: codeLines.join("\n") });
      continue;
    }

    if (/^[-*+]\s+/.test(trimmed)) {
      const items: string[] = [];
      while (index < lines.length) {
        const item = /^[-*+]\s+(.+)$/.exec(lines[index].trim());
        if (!item) {
          break;
        }
        items.push(item[1]);
        index += 1;
      }
      blocks.push({ kind: "unordered_list", items });
      continue;
    }

    if (/^\d+\.\s+/.test(trimmed)) {
      const items: string[] = [];
      while (index < lines.length) {
        const item = /^\d+\.\s+(.+)$/.exec(lines[index].trim());
        if (!item) {
          break;
        }
        items.push(item[1]);
        index += 1;
      }
      blocks.push({ kind: "ordered_list", items });
      continue;
    }

    const paragraphLines: string[] = [];
    while (index < lines.length && lines[index].trim() !== "") {
      if (startsBlock(lines[index]) && paragraphLines.length > 0) {
        break;
      }
      paragraphLines.push(lines[index].trim());
      index += 1;
    }
    blocks.push({
      kind: "paragraph",
      text: paragraphLines.join(" "),
    });
  }

  return blocks;
}

function startsBlock(line: string): boolean {
  const trimmed = line.trim();

  return (
    /^(#{1,6})\s+/.test(trimmed) ||
    trimmed === "$$" ||
    trimmed.startsWith("```") ||
    /^[-*+]\s+/.test(trimmed) ||
    /^\d+\.\s+/.test(trimmed)
  );
}

function parseInline(text: string): ReactNode[] {
  const nodes: ReactNode[] = [];
  let index = 0;

  while (index < text.length) {
    if (text[index] === "$" && text[index + 1] !== "$") {
      const end = findNext(text, "$", index + 1);
      if (end !== -1) {
        nodes.push(
          <KatexMath key={nodes.length} latex={text.slice(index + 1, end)} />,
        );
        index = end + 1;
        continue;
      }
    }

    if (text[index] === "`") {
      const end = text.indexOf("`", index + 1);
      if (end !== -1) {
        nodes.push(
          <code key={nodes.length}>{text.slice(index + 1, end)}</code>,
        );
        index = end + 1;
        continue;
      }
    }

    if (text.startsWith("**", index)) {
      const end = text.indexOf("**", index + 2);
      if (end !== -1) {
        nodes.push(
          <strong key={nodes.length}>
            {parseInline(text.slice(index + 2, end))}
          </strong>,
        );
        index = end + 2;
        continue;
      }
    }

    if (text[index] === "*") {
      const end = text.indexOf("*", index + 1);
      if (end !== -1) {
        nodes.push(
          <em key={nodes.length}>{parseInline(text.slice(index + 1, end))}</em>,
        );
        index = end + 1;
        continue;
      }
    }

    if (text[index] === "[") {
      const link = parseLink(text, index);
      if (link) {
        nodes.push(
          <a
            href={link.href}
            key={nodes.length}
            rel="noreferrer"
            target="_blank"
          >
            {parseInline(link.label)}
          </a>,
        );
        index = link.end;
        continue;
      }
    }

    const next = nextSpecialIndex(text, index + 1);
    nodes.push(text.slice(index, next));
    index = next;
  }

  return nodes;
}

function KatexMath({
  displayMode = false,
  latex,
}: {
  displayMode?: boolean;
  latex: string;
}) {
  const html = katex.renderToString(latex, {
    displayMode,
    strict: "ignore",
    throwOnError: false,
  });

  if (displayMode) {
    return (
      <div
        className={styles.markdownBlockMath}
        dangerouslySetInnerHTML={{ __html: html }}
      />
    );
  }

  return (
    <span
      className={styles.markdownInlineMath}
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}

function parseLink(
  text: string,
  start: number,
): { label: string; href: string; end: number } | null {
  const labelEnd = text.indexOf("]", start + 1);
  if (labelEnd === -1 || text[labelEnd + 1] !== "(") {
    return null;
  }

  const hrefEnd = text.indexOf(")", labelEnd + 2);
  if (hrefEnd === -1) {
    return null;
  }

  const href = text.slice(labelEnd + 2, hrefEnd).trim();
  if (!isSafeHref(href)) {
    return null;
  }

  return {
    label: text.slice(start + 1, labelEnd),
    href,
    end: hrefEnd + 1,
  };
}

function isSafeHref(href: string): boolean {
  return /^(https?:|mailto:)/i.test(href);
}

function nextSpecialIndex(text: string, start: number): number {
  let next = text.length;
  for (const token of ["$", "`", "*", "["]) {
    const index = text.indexOf(token, start);
    if (index !== -1 && index < next) {
      next = index;
    }
  }

  return next;
}

function findNext(text: string, token: string, start: number): number {
  let index = start;

  while (index < text.length) {
    index = text.indexOf(token, index);
    if (index === -1) {
      return -1;
    }

    if (index === 0 || text[index - 1] !== "\\") {
      return index;
    }

    index += token.length;
  }

  return -1;
}

function normalizeNewlines(text: string): string {
  return text.replace(/\r\n?/g, "\n");
}
