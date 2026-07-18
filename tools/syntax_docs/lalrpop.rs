//! Reads `src/frontend/formulation/grammar.lalrpop`.
//!
//! LALRPOP files are not Rust, so `syn` cannot read them. This is a small scanner
//! that extracts only the two things the documentation needs:
//!
//! * the terminal table declared by `extern { enum Token { "tok" => Token::Variant } }`;
//! * each nonterminal's `Name: Type = { alt, alt };` definition, reduced to its symbol
//!   sequences with the Rust action code after `=>` discarded.
//!
//! A nonterminal's declared result type is what links a production to the AST type it
//! builds, which is what makes grammar/AST drift visible in the generated document.
//!
//! The scanner is string-literal aware (the grammar contains terminals such as `"\\"`
//! and `"\\."`) and fails loudly on anything it does not recognize rather than
//! silently skipping a production.

use super::model::{Nonterminal, Production, Terminal};

pub struct Grammar {
    pub terminals: Vec<Terminal>,
    pub nonterminals: Vec<Nonterminal>,
}

pub fn extract(source: &str) -> Result<Grammar, String> {
    let text = strip_comments(source);
    let chars: Vec<char> = text.chars().collect();
    let mut scanner = Scanner { chars, pos: 0 };

    let mut terminals = Vec::new();
    let mut nonterminals = Vec::new();

    loop {
        scanner.skip_trivia();
        if scanner.eof() {
            break;
        }
        // `use ..;` and `grammar;` are declarations, not productions.
        if scanner.eat_word("use") || scanner.eat_word("grammar") {
            scanner.skip_past(';')?;
        } else if scanner.eat_word("extern") {
            let block = scanner.brace_block()?;
            terminals = parse_terminals(&block)?;
        } else if scanner.eat_word("match") {
            // Lexer precedence directive: skip its block(s); it declares no productions.
            scanner.brace_block()?;
            scanner.skip_trivia();
            while scanner.eat_word("else") {
                scanner.brace_block()?;
                scanner.skip_trivia();
            }
            scanner.skip_trivia();
            scanner.eat_char(';');
        } else {
            nonterminals.push(scanner.nonterminal()?);
        }
    }

    if terminals.is_empty() {
        return Err("grammar.lalrpop: no `extern { enum Token { .. } }` terminals found".to_owned());
    }
    if nonterminals.is_empty() {
        return Err("grammar.lalrpop: no nonterminals found".to_owned());
    }

    Ok(Grammar {
        terminals,
        nonterminals,
    })
}

/// Removes `//` line comments, ignoring `//` inside string literals.
fn strip_comments(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut index = 0;
    let mut in_string = false;
    while index < chars.len() {
        let ch = chars[index];
        if in_string {
            out.push(ch);
            if ch == '\\' && index + 1 < chars.len() {
                out.push(chars[index + 1]);
                index += 2;
                continue;
            }
            if ch == '"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        if ch == '"' {
            in_string = true;
            out.push(ch);
            index += 1;
            continue;
        }
        if ch == '/' && index + 1 < chars.len() && chars[index + 1] == '/' {
            while index < chars.len() && chars[index] != '\n' {
                index += 1;
            }
            continue;
        }
        out.push(ch);
        index += 1;
    }
    out
}

/// Parses `enum Token { "tok" => Token::Variant, "x" => Token::Y(<String>), }`.
fn parse_terminals(extern_block: &str) -> Result<Vec<Terminal>, String> {
    let Some(start) = extern_block.find("enum Token") else {
        return Err("grammar.lalrpop: `extern` block has no `enum Token`".to_owned());
    };
    let chars: Vec<char> = extern_block[start..].chars().collect();
    let mut scanner = Scanner { chars, pos: 0 };
    scanner.eat_word("enum");
    scanner.skip_trivia();
    scanner.eat_word("Token");
    let body = scanner.brace_block()?;

    let mut out = Vec::new();
    for entry in split_top_level(&body, ',') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        let Some((left, right)) = split_once_top_level(entry, "=>") else {
            return Err(format!("grammar.lalrpop: bad terminal entry `{entry}`"));
        };
        let name = unquote(left.trim()).ok_or_else(|| {
            format!("grammar.lalrpop: terminal `{}` is not a string literal", left.trim())
        })?;
        let right = right.trim();
        let (variant, payload) = match right.find('(') {
            Some(paren) => {
                let variant = right[..paren].trim().to_owned();
                let inner = right[paren + 1..].trim_end_matches(')').trim();
                // Payloads are written `<String>` / `<Vec<String>>`.
                let payload = inner
                    .strip_prefix('<')
                    .and_then(|i| i.strip_suffix('>'))
                    .map(|i| i.trim().to_owned());
                (variant, payload)
            }
            None => (right.to_owned(), None),
        };
        out.push(Terminal {
            name,
            variant,
            payload,
        });
    }
    Ok(out)
}

struct Scanner {
    chars: Vec<char>,
    pos: usize,
}

impl Scanner {
    fn eof(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn skip_trivia(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    /// Consumes `word` when it appears next as a whole word.
    fn eat_word(&mut self, word: &str) -> bool {
        self.skip_trivia();
        let end = self.pos + word.chars().count();
        if end > self.chars.len() {
            return false;
        }
        if self.chars[self.pos..end].iter().collect::<String>() != word {
            return false;
        }
        if let Some(next) = self.chars.get(end)
            && (next.is_alphanumeric() || *next == '_')
        {
            return false;
        }
        self.pos = end;
        true
    }

    fn eat_char(&mut self, ch: char) -> bool {
        self.skip_trivia();
        if self.chars.get(self.pos) == Some(&ch) {
            self.pos += 1;
            return true;
        }
        false
    }

    fn skip_past(&mut self, ch: char) -> Result<(), String> {
        while self.pos < self.chars.len() {
            if self.chars[self.pos] == ch {
                self.pos += 1;
                return Ok(());
            }
            self.pos += 1;
        }
        Err(format!("grammar.lalrpop: expected `{ch}` before end of file"))
    }

    /// Consumes a `{ .. }` block and returns its inner text.
    fn brace_block(&mut self) -> Result<String, String> {
        self.skip_trivia();
        if self.chars.get(self.pos) != Some(&'{') {
            return Err(format!(
                "grammar.lalrpop: expected `{{` at `{}`",
                self.peek_context()
            ));
        }
        let start = self.pos + 1;
        let mut depth = 0usize;
        let mut in_string = false;
        while self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            if in_string {
                if ch == '\\' {
                    self.pos += 2;
                    continue;
                }
                if ch == '"' {
                    in_string = false;
                }
                self.pos += 1;
                continue;
            }
            match ch {
                '"' => in_string = true,
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        let inner: String = self.chars[start..self.pos].iter().collect();
                        self.pos += 1;
                        return Ok(inner);
                    }
                }
                _ => {}
            }
            self.pos += 1;
        }
        Err("grammar.lalrpop: unbalanced `{`".to_owned())
    }

    /// Parses `pub? Name<Params>?: Type = { alternatives };`.
    fn nonterminal(&mut self) -> Result<Nonterminal, String> {
        self.skip_trivia();
        let public = self.eat_word("pub");
        let header_start = self.pos;

        // The header runs up to the `=` that introduces the body.
        let mut equals = None;
        let mut index = self.pos;
        while index < self.chars.len() {
            let ch = self.chars[index];
            if ch == '=' && self.chars.get(index + 1) != Some(&'>') {
                let mut probe = index + 1;
                while probe < self.chars.len() && self.chars[probe].is_whitespace() {
                    probe += 1;
                }
                if self.chars.get(probe) == Some(&'{') {
                    equals = Some(index);
                    break;
                }
            }
            if ch == ';' {
                break;
            }
            index += 1;
        }
        let Some(equals) = equals else {
            return Err(format!(
                "grammar.lalrpop: could not find `= {{` for the item at `{}`",
                self.peek_context()
            ));
        };

        let header: String = self.chars[header_start..equals].iter().collect();
        let (name, result_type) = split_header(&header)?;
        self.pos = equals + 1;
        let body = self.brace_block()?;
        self.skip_trivia();
        self.eat_char(';');

        let mut productions = Vec::new();
        for alternative in split_top_level(&body, ',') {
            let alternative = alternative.trim();
            if alternative.is_empty() {
                continue;
            }
            // Everything after the first top-level `=>` is Rust action code.
            let symbol_text = match split_once_top_level(alternative, "=>") {
                Some((symbols, _action)) => symbols.to_owned(),
                None => alternative.to_owned(),
            };
            productions.push(Production {
                symbols: parse_symbols(&symbol_text),
            });
        }

        let result_ast_type = ast_type_of(&result_type);
        Ok(Nonterminal {
            name,
            public,
            result_type,
            result_ast_type,
            productions,
        })
    }

    fn peek_context(&self) -> String {
        let end = (self.pos + 40).min(self.chars.len());
        self.chars[self.pos..end]
            .iter()
            .collect::<String>()
            .replace('\n', " ")
    }
}

/// Splits `Name<T>: ast::Type` into its name and result type.
fn split_header(header: &str) -> Result<(String, String), String> {
    let chars: Vec<char> = header.chars().collect();
    let mut depth = 0i32;
    for (index, ch) in chars.iter().enumerate() {
        match ch {
            '<' => depth += 1,
            '>' => depth -= 1,
            ':' if depth == 0 && chars.get(index + 1) != Some(&':') => {
                let name: String = chars[..index].iter().collect();
                let ty: String = chars[index + 1..].iter().collect();
                // Drop any macro parameter list: `Comma<T>` documents as `Comma`.
                let name = name.trim().split('<').next().unwrap_or("").trim().to_owned();
                if name.is_empty() {
                    return Err(format!("grammar.lalrpop: unnamed nonterminal in `{header}`"));
                }
                return Ok((name, ty.trim().to_owned()));
            }
            _ => {}
        }
    }
    Err(format!(
        "grammar.lalrpop: nonterminal header `{}` has no `: Type`",
        header.trim()
    ))
}

/// Reduces a declared result type to the bare AST type it builds, when it is one.
/// `ast::Expression` -> `Expression`; `Vec<ast::Placeholder>` and `String` -> `None`.
fn ast_type_of(result_type: &str) -> Option<String> {
    let trimmed = result_type.trim();
    let bare = trimmed.strip_prefix("ast::")?;
    if bare.contains('<') || bare.contains(' ') {
        return None;
    }
    Some(bare.to_owned())
}

/// Splits text at a delimiter appearing outside brackets and string literals.
fn split_top_level(text: &str, delimiter: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if in_string {
            current.push(ch);
            if ch == '\\' {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
                continue;
            }
            if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => {
                in_string = true;
                current.push(ch);
            }
            '(' | '[' | '{' => {
                depth += 1;
                current.push(ch);
            }
            ')' | ']' | '}' => {
                depth -= 1;
                current.push(ch);
            }
            _ if ch == delimiter && depth == 0 => {
                out.push(std::mem::take(&mut current));
            }
            _ => current.push(ch),
        }
    }
    out.push(current);
    out
}

/// Splits at the first occurrence of `needle` outside brackets and string literals.
fn split_once_top_level<'a>(text: &'a str, needle: &str) -> Option<(&'a str, &'a str)> {
    let bytes: Vec<char> = text.chars().collect();
    let needle_chars: Vec<char> = needle.chars().collect();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut index = 0;
    while index < bytes.len() {
        let ch = bytes[index];
        if in_string {
            if ch == '\\' {
                index += 2;
                continue;
            }
            if ch == '"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        match ch {
            '"' => in_string = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            _ => {}
        }
        if depth == 0 && bytes[index..].starts_with(needle_chars.as_slice()) {
            let byte_index: usize = bytes[..index].iter().map(|c| c.len_utf8()).sum();
            let needle_len: usize = needle.len();
            return Some((&text[..byte_index], &text[byte_index + needle_len..]));
        }
        index += 1;
    }
    None
}

/// Turns an alternative's symbol text into an ordered symbol list, unwrapping
/// `<name:Symbol>` bindings and dropping `@L`/`@R` position captures.
fn parse_symbols(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        if ch.is_whitespace() {
            index += 1;
            continue;
        }
        match ch {
            // `<name:Symbol>` — keep only the bound symbol.
            '<' => {
                let (inner, next) = read_balanced(&chars, index, '<', '>');
                index = next;
                let inner = inner.trim();
                let symbol = match split_once_top_level(inner, ":") {
                    Some((_binding, symbol)) => symbol.trim().to_owned(),
                    None => inner.to_owned(),
                };
                if let Some(symbol) = normalize_symbol(&symbol) {
                    out.push(symbol);
                }
            }
            // A quoted terminal.
            '"' => {
                let start = index;
                index += 1;
                while index < chars.len() {
                    if chars[index] == '\\' {
                        index += 2;
                        continue;
                    }
                    if chars[index] == '"' {
                        index += 1;
                        break;
                    }
                    index += 1;
                }
                let literal: String = chars[start..index].iter().collect();
                let mut symbol = literal;
                // A repetition/option suffix binds to the symbol.
                if let Some(next) = chars.get(index)
                    && matches!(next, '?' | '*' | '+')
                {
                    symbol.push(*next);
                    index += 1;
                }
                out.push(symbol);
            }
            // A parenthesized group, kept verbatim.
            '(' => {
                let (inner, next) = read_balanced(&chars, index, '(', ')');
                index = next;
                let mut symbol = format!("({})", inner.trim());
                if let Some(next) = chars.get(index)
                    && matches!(next, '?' | '*' | '+')
                {
                    symbol.push(*next);
                    index += 1;
                }
                out.push(symbol);
            }
            _ => {
                let start = index;
                while index < chars.len()
                    && !chars[index].is_whitespace()
                    && !matches!(chars[index], '<' | '"' | '(')
                {
                    index += 1;
                }
                let word: String = chars[start..index].iter().collect();
                if let Some(symbol) = normalize_symbol(&word) {
                    out.push(symbol);
                }
            }
        }
    }
    out
}

/// Drops position captures and empty text; keeps everything else as written.
fn normalize_symbol(symbol: &str) -> Option<String> {
    let trimmed = symbol.trim();
    if trimmed.is_empty() || trimmed == "@L" || trimmed == "@R" {
        return None;
    }
    Some(trimmed.to_owned())
}

/// Reads a bracketed run starting at `start`, returning the inner text and the index
/// just past the closing bracket.
fn read_balanced(chars: &[char], start: usize, open: char, close: char) -> (String, usize) {
    let mut depth = 0i32;
    let mut index = start;
    let mut in_string = false;
    while index < chars.len() {
        let ch = chars[index];
        if in_string {
            if ch == '\\' {
                index += 2;
                continue;
            }
            if ch == '"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        if ch == '"' {
            in_string = true;
        } else if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth == 0 {
                let inner: String = chars[start + 1..index].iter().collect();
                return (inner, index + 1);
            }
        }
        index += 1;
    }
    (chars[start + 1..].iter().collect(), chars.len())
}

/// Strips the surrounding quotes from a string literal, undoing `\` escapes.
fn unquote(text: &str) -> Option<String> {
    let inner = text.strip_prefix('"')?.strip_suffix('"')?;
    let mut out = String::new();
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some(next) => out.push(next),
                None => return None,
            }
        } else {
            out.push(ch);
        }
    }
    Some(out)
}
