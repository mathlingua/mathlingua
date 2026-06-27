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
};

interface MathLinguaSourceProps {
  source: string;
}

/** Renders MathLingua source with lightweight syntax coloring. */
export function MathLinguaSource({ source }: MathLinguaSourceProps) {
  const lines = source.length > 0 ? source.split("\n") : [""];

  return (
    <pre className={styles.source}>
      <code>
        {lines.map((line, lineIndex) => {
          const lineTokens = tokenizeLine(line);

          return (
            <span className={styles.line} key={`${lineIndex}-${line}`}>
              {lineTokens.map((token, tokenIndex) => (
                <span
                  className={classNameForToken(token.kind)}
                  key={`${tokenIndex}-${token.kind}-${token.text}`}
                >
                  {token.text}
                </span>
              ))}
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

function tokenizeInline(text: string): Token[] {
  const tokens: Token[] = [];
  let index = 0;

  while (index < text.length) {
    const char = text[index];

    if (isWhitespace(char)) {
      const end = scanWhile(text, index, isWhitespace);
      tokens.push({ kind: "plain", text: text.slice(index, end) });
      index = end;
      continue;
    }

    if (char === '"') {
      const end = scanString(text, index);
      tokens.push({ kind: "string", text: text.slice(index, end) });
      index = end;
      continue;
    }

    if (char === "\\") {
      const end = scanCommand(text, index);
      tokens.push({ kind: "command", text: text.slice(index, end) });
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

    tokens.push({ kind: "plain", text: char });
    index += 1;
  }

  return tokens;
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
  return /[A-Za-z0-9_.:/?\\]/.test(char);
}

function isPunctuation(char: string): boolean {
  return /[()[\]{},.;]/.test(char);
}
