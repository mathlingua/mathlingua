import styles from "./mathlingua-inline.module.css";

type InlineToken =
  | { kind: "text"; text: string }
  | { kind: "placeholder"; text: string; display: string }
  | { kind: "quotedOperator"; text: string; display: string }
  | { kind: "arrow"; text: string; display: string }
  | { kind: "operator"; text: string }
  | { kind: "command"; text: string }
  | { kind: "keyword"; text: string; display: string };

interface MathLinguaInlineProps {
  text: string;
  className?: string;
}

const arrowDisplays = new Map([
  [":->", "→"],
  [":=>", "⇒"],
  [":~>", "⇝"],
]);

const styledOperators = [
  "::=",
  ":=",
  "=",
  "!=",
  "<=",
  ">=",
  "+",
  "-",
  "*",
  "/",
  "^",
];

const keywords = new Map([
  ["is", "is"],
  ["via", "via"],
  ["member_of", "member_of"],
]);

/** Renders raw MathLingua fallback text with lightweight mathematical styling. */
export function MathLinguaInline({ text, className }: MathLinguaInlineProps) {
  const combinedClassName = className
    ? `${styles.inline} ${className}`
    : styles.inline;

  return (
    <span className={combinedClassName}>
      {tokenize(text).map((token, index) => renderToken(token, index))}
    </span>
  );
}

function renderToken(token: InlineToken, index: number) {
  switch (token.kind) {
    case "placeholder":
      return (
        <span className={styles.placeholder} key={index} title={token.text}>
          {token.display}
        </span>
      );
    case "quotedOperator":
      return (
        <span className={styles.quotedOperator} key={index} title={token.text}>
          {token.display}
        </span>
      );
    case "arrow":
      return (
        <span className={styles.arrow} key={index} title={token.text}>
          <span className={styles.arrowColon}>:</span>
          <span className={styles.arrowGlyph}>{token.display}</span>
        </span>
      );
    case "operator":
      return (
        <span className={styles.operator} key={index}>
          {token.text}
        </span>
      );
    case "command":
      return (
        <span className={styles.command} key={index}>
          {token.text}
        </span>
      );
    case "keyword":
      return (
        <span
          className={
            token.text === "member_of"
              ? `${styles.keyword} ${styles.memberOf}`
              : styles.keyword
          }
          key={index}
          title={token.text}
        >
          {token.display}
        </span>
      );
    case "text":
      return token.text;
  }
}

function tokenize(text: string): InlineToken[] {
  const tokens: InlineToken[] = [];
  let index = 0;

  while (index < text.length) {
    const arrow = scanArrow(text, index);
    if (arrow) {
      tokens.push({
        kind: "arrow",
        text: arrow,
        display: arrowDisplays.get(arrow) ?? arrow,
      });
      index += arrow.length;
      continue;
    }

    const command = scanCommand(text, index);
    if (command) {
      tokens.push({ kind: "command", text: command });
      index += command.length;
      continue;
    }

    const quotedOperator = scanQuotedOperator(text, index);
    if (quotedOperator) {
      tokens.push({
        kind: "quotedOperator",
        text: quotedOperator,
        display: quotedOperator.slice(1, -1),
      });
      index += quotedOperator.length;
      continue;
    }

    const placeholder = scanPlaceholder(text, index);
    if (placeholder) {
      tokens.push({
        kind: "placeholder",
        text: placeholder,
        display: placeholder.replace(/_+$/, ""),
      });
      index += placeholder.length;
      continue;
    }

    const keyword = scanKeyword(text, index);
    if (keyword) {
      tokens.push({
        kind: "keyword",
        text: keyword,
        display: keywords.get(keyword) ?? keyword,
      });
      index += keyword.length;
      continue;
    }

    const operator = scanOperator(text, index);
    if (operator) {
      tokens.push({ kind: "operator", text: operator });
      index += operator.length;
      continue;
    }

    tokens.push({ kind: "text", text: text[index] });
    index += 1;
  }

  return coalesceText(tokens);
}

function scanArrow(text: string, index: number): string | null {
  for (const arrow of arrowDisplays.keys()) {
    if (text.startsWith(arrow, index)) {
      return arrow;
    }
  }

  return null;
}

function scanCommand(text: string, index: number): string | null {
  if (text[index] !== "\\") {
    return null;
  }

  if (text.startsWith("\\.", index)) {
    const end = text.indexOf("./", index + 2);
    return end === -1 ? text.slice(index) : text.slice(index, end + 2);
  }

  let end = index + 1;
  if (text[end] === "\\") {
    end += 1;
  }

  while (end < text.length && /[A-Za-z0-9._?:]/.test(text[end])) {
    end += 1;
  }

  return text.slice(index, end);
}

function scanQuotedOperator(text: string, index: number): string | null {
  if (text[index] !== '"') {
    return null;
  }

  const end = text.indexOf('"', index + 1);

  return end === -1 ? null : text.slice(index, end + 1);
}

function scanPlaceholder(text: string, index: number): string | null {
  const match = /^[A-Za-z][A-Za-z0-9]*_{1,2}/.exec(text.slice(index));

  if (!match) {
    return null;
  }

  const before = index === 0 ? "" : text[index - 1];
  const after = text[index + match[0].length] ?? "";

  if (/[A-Za-z0-9_]/.test(before) || /[A-Za-z0-9_]/.test(after)) {
    return null;
  }

  return match[0];
}

function scanKeyword(text: string, index: number): string | null {
  for (const keyword of keywords.keys()) {
    if (!text.startsWith(keyword, index)) {
      continue;
    }

    const before = index === 0 ? "" : text[index - 1];
    const after = text[index + keyword.length] ?? "";

    if (!/[A-Za-z0-9_]/.test(before) && !/[A-Za-z0-9_]/.test(after)) {
      return keyword;
    }
  }

  return null;
}

function scanOperator(text: string, index: number): string | null {
  for (const operator of styledOperators) {
    if (text.startsWith(operator, index)) {
      return operator;
    }
  }

  const match = /^[!<>=+\-*/^|&~]+/.exec(text.slice(index));

  return match ? match[0] : null;
}

function coalesceText(tokens: InlineToken[]): InlineToken[] {
  const coalesced: InlineToken[] = [];

  for (const token of tokens) {
    const last = coalesced[coalesced.length - 1];
    if (token.kind === "text" && last?.kind === "text") {
      last.text += token.text;
    } else {
      coalesced.push(token);
    }
  }

  return coalesced;
}
