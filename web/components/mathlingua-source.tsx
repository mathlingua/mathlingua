"use client";

import styles from "./mathlingua-source.module.css";

type TokenKind =
  | "plain"
  | "comment"
  | "command"
  | "keyword"
  | "label"
  | "operator"
  | "placeholder"
  | "punctuation"
  | "string";

type Token = {
  kind: TokenKind;
  text: string;
  referenceKey?: string;
};

interface MathLinguaSourceProps {
  source: string;
  /** Opens the definition card for a referenced command token. */
  onReferenceClick?: (referenceKey: string) => void;
}

/** Renders MathLingua source with lightweight syntax coloring. */
export function MathLinguaSource({
  source,
  onReferenceClick,
}: MathLinguaSourceProps) {
  const lines = source.length > 0 ? source.split("\n") : [""];

  return (
    <pre className={styles.source}>
      <code>
        {lines.map((line, lineIndex) => {
          const lineTokens = tokenizeLine(line);

          return (
            <span className={styles.line} key={`${lineIndex}-${line}`}>
              {lineTokens.map((token, tokenIndex) => {
                const className = classNameForToken(token.kind);
                const key = `${tokenIndex}-${token.kind}-${token.text}`;

                return token.referenceKey && onReferenceClick ? (
                  <button
                    aria-label={`Show definition for ${token.text}`}
                    className={`${className} ${styles.reference}`}
                    data-mlg-ref={token.referenceKey}
                    key={key}
                    onClick={() => onReferenceClick(token.referenceKey!)}
                    title={`Show definition for ${token.text}`}
                    type="button"
                  >
                    {token.text}
                  </button>
                ) : (
                  <span className={className} key={key}>
                    {token.text}
                  </span>
                );
              })}
              {lineIndex < lines.length - 1 ? "\n" : null}
            </span>
          );
        })}
      </code>
    </pre>
  );
}

function classNameForToken(kind: TokenKind): string {
  switch (kind) {
    case "comment":
      return styles.comment;
    case "command":
      return styles.command;
    case "keyword":
      return styles.keyword;
    case "label":
      return styles.label;
    case "operator":
      return styles.operator;
    case "placeholder":
      return styles.placeholder;
    case "punctuation":
      return styles.punctuation;
    case "string":
      return styles.string;
    case "plain":
    default:
      return styles.plain;
  }
}

function tokenizeLine(line: string): Token[] {
  if (line.trimStart().startsWith("--")) {
    return [{ kind: "comment", text: line }];
  }

  const sectionMatch = line.match(
    /^(\s*(?:\.\s*)?)([A-Za-z_][A-Za-z0-9_]*)(:)/,
  );
  if (sectionMatch) {
    const [, prefix, label, colon] = sectionMatch;
    const rest = line.slice(prefix.length + label.length + colon.length);

    return [
      { kind: "plain", text: prefix },
      { kind: "label", text: label },
      { kind: "punctuation", text: colon },
      ...tokenizeInline(rest),
    ];
  }

  return tokenizeInline(line);
}

function tokenizeInline(text: string, insideString = false): Token[] {
  const tokens: Token[] = [];
  let index = 0;
  const plainKind: TokenKind = insideString ? "string" : "plain";

  while (index < text.length) {
    const char = text[index];

    if (isWhitespace(char)) {
      const end = scanWhile(text, index, isWhitespace);
      tokens.push({ kind: plainKind, text: text.slice(index, end) });
      index = end;
      continue;
    }

    if (char === '"') {
      const end = scanString(text, index);
      const hasClosingQuote = end > index + 1 && text[end - 1] === '"';
      const contentEnd = hasClosingQuote ? end - 1 : end;
      tokens.push({ kind: "string", text: '"' });
      tokens.push(...tokenizeInline(text.slice(index + 1, contentEnd), true));
      if (hasClosingQuote) {
        tokens.push({ kind: "string", text: '"' });
      }
      index = end;
      continue;
    }

    if (char === "\\") {
      const end = scanCommand(text, index);
      const signature = commandReferenceSignature(text, index, end);
      tokens.push({
        kind: "command",
        text: text.slice(index, end),
        referenceKey: encodeReferenceKey(signature),
      });
      index = end;
      continue;
    }

    const operator = operatorAt(text, index);
    if (operator) {
      tokens.push({ kind: "operator", text: operator });
      index += operator.length;
      continue;
    }

    if (isPunctuation(char)) {
      tokens.push({ kind: "punctuation", text: char });
      index += 1;
      continue;
    }

    if (isWordStart(char)) {
      const end = scanWhile(text, index, isWordPart);
      const word = text.slice(index, end);
      tokens.push({
        kind: tokenKindForWord(word),
        text: word,
      });
      index = end;
      continue;
    }

    tokens.push({ kind: plainKind, text: char });
    index += 1;
  }

  return tokens;
}

/** Reconstructs a command's definition signature from its named arguments. */
function commandReferenceSignature(
  text: string,
  start: number,
  commandEnd: number,
): string {
  let signature = text.slice(start, commandEnd);
  let cursor = commandEnd;

  while (text[cursor] === "{") {
    cursor = scanBalancedGroup(text, cursor);
    if (text[cursor] !== ":") {
      break;
    }

    let labelEnd = cursor + 1;
    while (labelEnd < text.length && /[A-Za-z0-9_.?]/.test(text[labelEnd])) {
      labelEnd += 1;
    }
    if (labelEnd === cursor + 1) {
      break;
    }
    signature += text.slice(cursor, labelEnd);
    cursor = labelEnd;
  }

  return signature;
}

function scanBalancedGroup(text: string, start: number): number {
  let depth = 0;
  for (let index = start; index < text.length; index += 1) {
    if (text[index] === "{") {
      depth += 1;
    } else if (text[index] === "}") {
      depth -= 1;
      if (depth === 0) {
        return index + 1;
      }
    }
  }
  return text.length;
}

function encodeReferenceKey(signature: string): string {
  return Array.from(new TextEncoder().encode(signature), (byte) =>
    byte.toString(16).padStart(2, "0"),
  ).join("");
}

function scanString(text: string, start: number): number {
  for (let index = start + 1; index < text.length; index += 1) {
    if (text[index] === '"') {
      return index + 1;
    }
  }

  return text.length;
}

function scanCommand(text: string, start: number): number {
  let index = start + 1;

  while (index < text.length && isCommandPart(text[index])) {
    index += 1;
  }

  // A dot may be part of a dotted command name, but a final dot before prose
  // punctuation or a closing formulation delimiter belongs to the sentence.
  while (index > start + 1 && text[index - 1] === ".") {
    index -= 1;
  }

  return index;
}

function scanWhile(
  text: string,
  start: number,
  predicate: (char: string) => boolean,
): number {
  let index = start;

  while (index < text.length && predicate(text[index])) {
    index += 1;
  }

  return index;
}

function operatorAt(text: string, index: number): string | null {
  for (const operator of [
    "::=",
    ":<->:",
    ":->",
    ":=>",
    ":~>",
    ":?",
    ":=",
    "!=",
    "==",
    "<=",
    ">=",
    "=>",
    "->",
    "::",
  ]) {
    if (text.startsWith(operator, index)) {
      return operator;
    }
  }

  return /^[=+\-*/|<>?!]$/.test(text[index]) ? text[index] : null;
}

function tokenKindForWord(word: string): TokenKind {
  if (word.endsWith("_")) {
    return "placeholder";
  }

  if (["is", "via", "in", "not"].includes(word)) {
    return "keyword";
  }

  return "plain";
}

function isWhitespace(char: string): boolean {
  return /\s/.test(char);
}

function isWordStart(char: string): boolean {
  return /[A-Za-z_]/.test(char);
}

function isWordPart(char: string): boolean {
  return /[A-Za-z0-9_.]/.test(char);
}

function isCommandPart(char: string): boolean {
  return /[A-Za-z0-9_.:?\\+\-*|<>=!]/.test(char) || char === "/";
}

function isPunctuation(char: string): boolean {
  return /[()[\]{},.;]/.test(char);
}
