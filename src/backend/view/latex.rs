use super::builder::build_single_group_view;
use super::render::{
    RenderRegistry, markdown_code_span_len, math_fragment, render_builtin_command_expression,
    render_command_expression, render_formulation_latex, render_infix_command_expression,
    render_written_display_template, theorem_reference_fragment,
};
use crate::backend::collection::{SourceCollection, SourceFileFilter};
use crate::events::{Event, EventLocation, EventLog, Level};
use crate::frontend::formulation::ast::{
    Chain, ChainPart, CurlyExpressionArgs, Expression, ExpressionKind, ParenExpressionArgs,
    Span, TupleExpressionElement, TypeExpression,
};
use crate::frontend::formulation::{parse_expression, parse_type_expression};
use crate::frontend::proto::ast::{Argument, Group, Section};
use crate::frontend::{ParsedSourceFile, ProtoParser};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

/// Validates whether a rendered LaTeX string can be parsed by KaTeX.
pub fn validate_latex(latex: &str) -> Result<(), String> {
    let trimmed = latex.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    if let Ok(cache) = KATEX_CACHE.lock() {
        if let Some(res) = cache.get(trimmed) {
            return res.clone();
        }
    }

    let opts = katex::Opts::builder()
        .display_mode(false)
        .throw_on_error(true)
        .trust(true)
        .build()
        .unwrap();

    let result = match katex::render_with_opts(trimmed, &opts) {
        Ok(_) => Ok(()),
        Err(err) => Err(clean_katex_error(&err)),
    };

    if let Ok(mut cache) = KATEX_CACHE.lock() {
        cache.insert(trimmed.to_string(), result.clone());
    }

    result
}

/// Checks all files in the collection for LaTeX errors that would occur during an `mlg view`.
pub(crate) fn check_collection_latex(
    collection: &SourceCollection,
    render_registry: &RenderRegistry,
    filter: &SourceFileFilter,
    event_log: &mut EventLog,
    origin: &str,
) {
    let mut seen = HashSet::new();

    for file in collection.all_parsed_files() {
        check_file_latex(file, render_registry, filter, event_log, origin, &mut seen);
    }
}

// =============================================================================

static KATEX_CACHE: LazyLock<Mutex<HashMap<String, Result<(), String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

enum TextMathFragment<'a> {
    MathLingua {
        source: &'a str,
        start_offset: usize,
        #[allow(dead_code)]
        is_display: bool,
    },
    TheoremRef {
        source: &'a str,
        start_offset: usize,
    },
    RawLatex {
        latex: &'a str,
        start_offset: usize,
        #[allow(dead_code)]
        is_display: bool,
    },
}

#[derive(Clone, Debug)]
struct FoundCommand {
    name: String,
    span: Span,
    rendered_latex: Option<String>,
}

struct SourceLocator<'a> {
    source: &'a str,
    line_offsets: Vec<usize>,
}

impl<'a> SourceLocator<'a> {
    fn new(source: &'a str) -> Self {
        let mut line_offsets = vec![0];
        for (i, b) in source.bytes().enumerate() {
            if b == b'\n' {
                line_offsets.push(i + 1);
            }
        }
        Self { source, line_offsets }
    }

    fn line_offset(&self, row: usize) -> usize {
        self.line_offsets.get(row).copied().unwrap_or(0)
    }

    fn offset_to_row_col(&self, offset: usize) -> (usize, usize) {
        let offset = offset.min(self.source.len());
        let row = match self.line_offsets.binary_search(&offset) {
            Ok(exact) => exact,
            Err(next) => next.saturating_sub(1),
        };
        let line_start = self.line_offsets[row];
        let col = offset.saturating_sub(line_start);
        (row, col)
    }

    fn find_on_or_after_row(&self, start_row: usize, needle: &str) -> Option<usize> {
        if needle.is_empty() {
            return None;
        }
        let start_offset = self.line_offsets.get(start_row).copied().unwrap_or(0);
        self.source[start_offset..]
            .find(needle)
            .map(|pos| start_offset + pos)
    }

    fn find_column_on_row(&self, row: usize, needle: &str) -> usize {
        if needle.is_empty() {
            return 0;
        }
        let line_start = self.line_offset(row);
        let line_end = self.line_offsets.get(row + 1).map(|&o| o.saturating_sub(1)).unwrap_or(self.source.len());
        if line_start >= self.source.len() {
            return 0;
        }
        let line = &self.source[line_start..line_end];
        line.find(needle).unwrap_or(0)
    }
}

fn clean_katex_error(err: &katex::Error) -> String {
    let msg = err.to_string();
    if let Some(pos) = msg.find("KaTeX parse error:") {
        msg[pos..].trim().to_string()
    } else {
        msg.trim().to_string()
    }
}

fn check_file_latex(
    file: &ParsedSourceFile,
    registry: &RenderRegistry,
    filter: &SourceFileFilter,
    event_log: &mut EventLog,
    origin: &str,
    seen: &mut HashSet<(PathBuf, usize, usize, String)>,
) {
    let locator = SourceLocator::new(&file.source);
    let mut proto_log = EventLog::new();
    let proto_groups = ProtoParser::new(&file.source, &mut proto_log).parse();

    for group in &proto_groups {
        check_group_latex(
            &file.path,
            group,
            &locator,
            registry,
            filter,
            event_log,
            origin,
            seen,
        );
    }
}

fn check_group_latex(
    path: &Path,
    group: &Group,
    locator: &SourceLocator<'_>,
    registry: &RenderRegistry,
    filter: &SourceFileFilter,
    event_log: &mut EventLog,
    origin: &str,
    seen: &mut HashSet<(PathBuf, usize, usize, String)>,
) {
    let group_view = build_single_group_view(group.clone(), String::new(), None, registry);

    // 1. Check heading LaTeX
    if let Some(ref heading_latex) = group_view.heading_latex {
        if let Err(err) = validate_latex(heading_latex) {
            let row = group.metadata.row;
            let cmd = extract_heading_command(group);
            let col = find_command_column_on_row(locator, row, cmd.as_deref());
            emit_latex_warning(
                path, row, col, cmd.as_deref(), &err, filter, event_log, origin, seen,
            );
        }
    }

    // 2. Check parameter destructurings
    for destructuring in &group_view.parameter_destructurings {
        if let Err(err) = validate_latex(destructuring) {
            let row = group.metadata.row;
            let cmd = extract_heading_command(group);
            let col = find_command_column_on_row(locator, row, cmd.as_deref());
            emit_latex_warning(
                path, row, col, cmd.as_deref(), &err, filter, event_log, origin, seen,
            );
        }
    }

    // 3. Check sections and arguments
    for section in &group.sections {
        check_section_latex(
            path, section, locator, registry, filter, event_log, origin, seen,
        );
    }
}

fn check_section_latex(
    path: &Path,
    section: &Section,
    locator: &SourceLocator<'_>,
    registry: &RenderRegistry,
    filter: &SourceFileFilter,
    event_log: &mut EventLog,
    origin: &str,
    seen: &mut HashSet<(PathBuf, usize, usize, String)>,
) {
    let row = section.metadata.row;

    // A. Check inline argument if present
    if let Some(ref inline_arg) = section.inline_argument {
        let trimmed = inline_arg.trim();
        if section.label == "written" {
            let unquoted = unquote_str(trimmed);
            let rendered = render_written_display_template(&unquoted);
            if let Err(err) = validate_latex(&rendered) {
                let col = locator.find_column_on_row(row, trimmed);
                emit_latex_warning(path, row, col, None, &err, filter, event_log, origin, seen);
            }
        } else if trimmed.starts_with('"') {
            // String argument, e.g. called: "..."
            check_text_string(
                path, trimmed, row, locator, registry, filter, event_log, origin, seen,
            );
        } else {
            // Inline formulation
            check_formulation_text(
                path, trimmed, row, locator, registry, filter, event_log, origin, seen,
            );
        }
    }

    // B. Check arguments
    for arg in &section.arguments {
        match arg {
            Argument::Formulation(formulation) => {
                check_formulation_text(
                    path,
                    &formulation.text,
                    formulation.metadata.row,
                    locator,
                    registry,
                    filter,
                    event_log,
                    origin,
                    seen,
                );
            }
            Argument::Text(text_literal) => {
                check_text_string(
                    path,
                    &text_literal.text,
                    text_literal.metadata.row,
                    locator,
                    registry,
                    filter,
                    event_log,
                    origin,
                    seen,
                );
            }
            Argument::Group(child_group) => {
                check_group_latex(
                    path,
                    child_group,
                    locator,
                    registry,
                    filter,
                    event_log,
                    origin,
                    seen,
                );
            }
        }
    }
}

fn check_formulation_text(
    path: &Path,
    formulation_text: &str,
    source_row: usize,
    locator: &SourceLocator<'_>,
    registry: &RenderRegistry,
    filter: &SourceFileFilter,
    event_log: &mut EventLog,
    origin: &str,
    seen: &mut HashSet<(PathBuf, usize, usize, String)>,
) {
    let Some(rendered_latex) = render_formulation_latex(formulation_text, registry) else {
        return;
    };

    let Err(full_err) = validate_latex(&rendered_latex) else {
        return;
    };

    // Locate the start of the formulation in file.source
    let first_line = formulation_text.lines().next().unwrap_or(formulation_text);
    let form_start_offset = locator
        .find_on_or_after_row(source_row, first_line)
        .unwrap_or_else(|| locator.line_offset(source_row));

    let commands = collect_commands(formulation_text, registry);

    if let Some(failing_cmd) = find_failing_command(formulation_text, &commands, registry) {
        // Compute exact row and column of the failing command
        let cmd_offset_in_form = failing_cmd.span.start;
        let cmd_file_offset = form_start_offset + cmd_offset_in_form;
        let (row, col) = locator.offset_to_row_col(cmd_file_offset);
        let err = if let Some(ref cmd_latex) = failing_cmd.rendered_latex {
            validate_latex(cmd_latex).err().unwrap_or(full_err)
        } else {
            full_err
        };
        emit_latex_warning(
            path,
            row,
            col,
            Some(&failing_cmd.name),
            &err,
            filter,
            event_log,
            origin,
            seen,
        );
    }
}

fn check_text_string(
    path: &Path,
    raw_text: &str,
    source_row: usize,
    locator: &SourceLocator<'_>,
    registry: &RenderRegistry,
    filter: &SourceFileFilter,
    event_log: &mut EventLog,
    origin: &str,
    seen: &mut HashSet<(PathBuf, usize, usize, String)>,
) {
    // Find text start in file.source
    let first_line = raw_text.lines().next().unwrap_or(raw_text);
    let text_start_offset = locator
        .find_on_or_after_row(source_row, first_line)
        .unwrap_or_else(|| locator.line_offset(source_row));

    let (content, content_offset) = if raw_text.starts_with("\"\"\"") && raw_text.ends_with("\"\"\"") && raw_text.len() >= 6 {
        (&raw_text[3..raw_text.len() - 3], 3)
    } else if raw_text.starts_with('"') && raw_text.ends_with('"') && raw_text.len() >= 2 {
        (&raw_text[1..raw_text.len() - 1], 1)
    } else {
        (raw_text, 0)
    };

    let fragments = scan_text_math_fragments(content);

    for fragment in fragments {
        match fragment {
            TextMathFragment::MathLingua { source, start_offset, .. } => {
                let trimmed = source.trim();
                let leading_ws = source.len() - source.trim_start().len();
                if let Some(latex) = render_formulation_latex(trimmed, registry) {
                    if let Err(full_err) = validate_latex(&latex) {
                        let commands = collect_commands(trimmed, registry);
                        if let Some(failing_cmd) = find_failing_command(trimmed, &commands, registry) {
                            let cmd_offset = start_offset + 2 + leading_ws + failing_cmd.span.start;
                            let file_offset = text_start_offset + content_offset + cmd_offset;
                            let (row, col) = locator.offset_to_row_col(file_offset);
                            let err = if let Some(ref cmd_latex) = failing_cmd.rendered_latex {
                                validate_latex(cmd_latex).err().unwrap_or(full_err)
                            } else {
                                full_err
                            };
                            emit_latex_warning(
                                path,
                                row,
                                col,
                                Some(&failing_cmd.name),
                                &err,
                                filter,
                                event_log,
                                origin,
                                seen,
                            );
                        } else {
                            let file_offset = text_start_offset + content_offset + start_offset;
                            let (row, col) = locator.offset_to_row_col(file_offset);
                            emit_latex_warning(
                                path, row, col, None, &full_err, filter, event_log, origin, seen,
                            );
                        }
                    }
                }
            }
            TextMathFragment::TheoremRef { source, start_offset } => {
                let trimmed = source.trim();
                let leading_ws = source.len() - source.trim_start().len();
                if let Some(latex) = render_formulation_latex(trimmed, registry) {
                    if let Err(full_err) = validate_latex(&latex) {
                        let commands = collect_commands(trimmed, registry);
                        let cmd_name = commands.first().map(|c| c.name.clone()).unwrap_or_else(|| {
                            format!("\\{}", trimmed.trim_start_matches('\\'))
                        });
                        let cmd_offset = start_offset + 2 + leading_ws;
                        let file_offset = text_start_offset + content_offset + cmd_offset;
                        let (row, col) = locator.offset_to_row_col(file_offset);
                        emit_latex_warning(
                            path,
                            row,
                            col,
                            Some(&cmd_name),
                            &full_err,
                            filter,
                            event_log,
                            origin,
                            seen,
                        );
                    }
                }
            }
            TextMathFragment::RawLatex { latex, start_offset, .. } => {
                if let Err(err) = validate_latex(latex) {
                    let file_offset = text_start_offset + content_offset + start_offset;
                    let (row, col) = locator.offset_to_row_col(file_offset);
                    emit_latex_warning(path, row, col, None, &err, filter, event_log, origin, seen);
                }
            }
        }
    }
}

fn emit_latex_warning(
    path: &Path,
    row: usize,
    col: usize,
    command: Option<&str>,
    err: &str,
    filter: &SourceFileFilter,
    event_log: &mut EventLog,
    origin: &str,
    seen: &mut HashSet<(PathBuf, usize, usize, String)>,
) {
    let message = match command {
        Some(cmd) => format!("For `{cmd}`: {err}"),
        None => err.to_string(),
    };

    let key = (path.to_path_buf(), row, col, message.clone());
    if seen.contains(&key) {
        return;
    }
    seen.insert(key);

    let location = EventLocation::file_row_and_column(path, row, col);
    let event = Event::message(message, Level::Warning, crate::events::Audience::User, Some(location))
        .with_origin_option(Some(origin));

    if filter.allows(&event) {
        event_log.push(event);
    }
}

fn extract_heading_command(group: &Group) -> Option<String> {
    if let Some(ref heading) = group.heading {
        let trimmed = heading.trim();
        let inner = trimmed
            .strip_prefix('[')
            .and_then(|s| s.strip_suffix(']'))
            .unwrap_or(trimmed);
        if let Some(pos) = inner.find('\\') {
            let after = &inner[pos + 1..];
            let end = after
                .find(|c: char| !c.is_alphanumeric() && c != '.' && c != '\'' && c != '_')
                .unwrap_or(after.len());
            return Some(format!("\\{}", &after[..end]));
        }
    }
    if let Some(first_sec) = group.sections.first() {
        if let Some(ref inline_arg) = first_sec.inline_argument {
            if let Some(pos) = inline_arg.find('\\') {
                let after = &inline_arg[pos + 1..];
                let end = after
                    .find(|c: char| !c.is_alphanumeric() && c != '.' && c != '\'' && c != '_')
                    .unwrap_or(after.len());
                return Some(format!("\\{}", &after[..end]));
            }
        }
    }
    None
}

fn find_command_column_on_row(locator: &SourceLocator<'_>, row: usize, cmd: Option<&str>) -> usize {
    if let Some(cmd) = cmd {
        locator.find_column_on_row(row, cmd)
    } else {
        0
    }
}

fn unquote_str(text: &str) -> String {
    let text = text.trim();
    if text.starts_with("\"\"\"") && text.ends_with("\"\"\"") && text.len() >= 6 {
        text[3..text.len() - 3].to_string()
    } else if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        text[1..text.len() - 1].to_string()
    } else {
        text.to_string()
    }
}

fn scan_text_math_fragments<'a>(text: &'a str) -> Vec<TextMathFragment<'a>> {
    let mut fragments = Vec::new();
    let mut index = 0;

    while index < text.len() {
        let rest = &text[index..];

        // 1. Skip code spans: `...`
        if let Some(code_len) = markdown_code_span_len(rest) {
            index += code_len;
            continue;
        }

        // 2. Check for display formulation: `{{.` ... `.}}`
        if let Some((source, consumed)) = math_fragment(rest, true) {
            fragments.push(TextMathFragment::MathLingua {
                source,
                start_offset: index,
                is_display: true,
            });
            index += consumed;
            continue;
        }

        // 3. Check for inline formulation: `{.` ... `.}`
        if let Some((source, consumed)) = math_fragment(rest, false) {
            fragments.push(TextMathFragment::MathLingua {
                source,
                start_offset: index,
                is_display: false,
            });
            index += consumed;
            continue;
        }

        // 4. Check for theorem ref: `{: ... :}`
        if let Some((source, consumed)) = theorem_reference_fragment(rest) {
            fragments.push(TextMathFragment::TheoremRef {
                source,
                start_offset: index,
            });
            index += consumed;
            continue;
        }

        // 5. Check for display raw LaTeX: `$$ ... $$`
        if rest.starts_with("$$") {
            let tail = &rest[2..];
            if let Some(end) = tail.find("$$") {
                let latex = &tail[..end];
                fragments.push(TextMathFragment::RawLatex {
                    latex,
                    start_offset: index,
                    is_display: true,
                });
                index += 2 + end + 2;
                continue;
            }
        }

        // 6. Check for inline raw LaTeX: `$ ... $` (not followed by another $)
        if rest.starts_with('$') && !rest.starts_with("$$") {
            let tail = &rest[1..];
            if let Some(end) = tail.find('$') {
                let latex = &tail[..end];
                fragments.push(TextMathFragment::RawLatex {
                    latex,
                    start_offset: index,
                    is_display: false,
                });
                index += 1 + end + 1;
                continue;
            }
        }

        index += rest.chars().next().map(char::len_utf8).unwrap_or(1);
    }

    fragments
}

fn chain_to_name(chain: &Chain) -> String {
    let mut parts = Vec::new();
    for part in &chain.parts {
        match part {
            ChainPart::Name(s) | ChainPart::Alias(s) | ChainPart::Operator(s) => {
                parts.push(s.as_str());
            }
        }
    }
    format!("\\{}", parts.join("."))
}

fn collect_commands(text: &str, registry: &RenderRegistry) -> Vec<FoundCommand> {
    let mut out = Vec::new();

    if let Ok(expr) = parse_expression(text) {
        collect_from_expression(&expr, registry, &mut out);
    } else if let Ok(ty) = parse_type_expression(text, true) {
        collect_from_type_expression(&ty, registry, &mut out);
    }

    // Fallback: scan for any `\command` directly in the formulation text
    scan_text_for_commands(text, &mut out);

    out
}

fn scan_text_for_commands(text: &str, out: &mut Vec<FoundCommand>) {
    let mut index = 0;
    while let Some(pos) = text[index..].find('\\') {
        let abs_pos = index + pos;
        let rest = &text[abs_pos + 1..];
        let len = rest
            .find(|c: char| !c.is_alphanumeric() && c != '.' && c != '\'' && c != '_')
            .unwrap_or(rest.len());
        if len > 0 {
            let cmd_name = format!("\\{}", &rest[..len]);
            let span = Span::new(abs_pos, abs_pos + 1 + len);
            if !out.iter().any(|c| c.span.start == span.start) {
                out.push(FoundCommand {
                    name: cmd_name,
                    span,
                    rendered_latex: None,
                });
            }
            index = abs_pos + 1 + len;
        } else {
            index = abs_pos + 1;
        }
    }
}

fn collect_from_type_expression(
    ty: &TypeExpression,
    registry: &RenderRegistry,
    out: &mut Vec<FoundCommand>,
) {
    match ty {
        TypeExpression::Command(cmd) => {
            out.push(FoundCommand {
                name: chain_to_name(&cmd.chain),
                span: cmd.span,
                rendered_latex: Some(render_command_expression(cmd, registry)),
            });
            for h in &cmd.head_args {
                collect_from_curly_args(h, registry, out);
            }
            for t in &cmd.tail {
                for a in &t.args {
                    collect_from_curly_args(a, registry, out);
                }
            }
            for p in &cmd.paren_args {
                collect_from_paren_args(p, registry, out);
            }
        }
        TypeExpression::RefinedCommand(ref_cmd) => {
            let name = ref_cmd
                .prefix_chain
                .as_ref()
                .map(chain_to_name)
                .unwrap_or_else(|| "\\refined".to_string());
            out.push(FoundCommand {
                name,
                span: ref_cmd.span,
                rendered_latex: None,
            });
            for h in &ref_cmd.head_args {
                collect_from_curly_args(h, registry, out);
            }
            for t in &ref_cmd.tail {
                for a in &t.args {
                    collect_from_curly_args(a, registry, out);
                }
            }
            for p in &ref_cmd.paren_args {
                collect_from_paren_args(p, registry, out);
            }
        }
        _ => {}
    }
}

fn collect_from_expression(
    expr: &Expression,
    registry: &RenderRegistry,
    out: &mut Vec<FoundCommand>,
) {
    match &expr.kind {
        ExpressionKind::Command(cmd) => {
            out.push(FoundCommand {
                name: chain_to_name(&cmd.chain),
                span: cmd.span,
                rendered_latex: Some(render_command_expression(cmd, registry)),
            });
            for h in &cmd.head_args {
                collect_from_curly_args(h, registry, out);
            }
            for t in &cmd.tail {
                for a in &t.args {
                    collect_from_curly_args(a, registry, out);
                }
            }
            for p in &cmd.paren_args {
                collect_from_paren_args(p, registry, out);
            }
        }
        ExpressionKind::InfixCommand { left, command, right } => {
            out.push(FoundCommand {
                name: chain_to_name(&command.chain),
                span: command.span,
                rendered_latex: Some(render_infix_command_expression(left, command, right, registry)),
            });
            collect_from_expression(left, registry, out);
            collect_from_expression(right, registry, out);
        }
        ExpressionKind::BuiltinCommand(cmd) => {
            let name = format!("\\\\{}", chain_to_name(&cmd.chain).trim_start_matches('\\'));
            out.push(FoundCommand {
                name,
                span: cmd.span,
                rendered_latex: Some(render_builtin_command_expression(cmd, registry)),
            });
        }
        ExpressionKind::Binary { left, right, .. } => {
            collect_from_expression(left, registry, out);
            collect_from_expression(right, registry, out);
        }
        ExpressionKind::Prefix { expression, .. } | ExpressionKind::Postfix { expression, .. } => {
            collect_from_expression(expression, registry, out);
        }
        ExpressionKind::Grouped { expression, .. } | ExpressionKind::Labeled { expression, .. } => {
            collect_from_expression(expression, registry, out);
        }
        ExpressionKind::Tuple(elements) => {
            for el in elements {
                if let TupleExpressionElement::Expression(e) = el {
                    collect_from_expression(e, registry, out);
                }
            }
        }
        ExpressionKind::FunctionCall { arguments, .. } => {
            for a in arguments {
                collect_from_expression(a, registry, out);
            }
        }
        ExpressionKind::FunctionNamedCall { elements, .. } => {
            for el in elements {
                collect_from_expression(&el.expression, registry, out);
            }
        }
        ExpressionKind::MemberCall { owner, arguments, .. } => {
            collect_from_expression(owner, registry, out);
            for a in arguments {
                collect_from_expression(a, registry, out);
            }
        }
        ExpressionKind::MemberAccess { owner, .. } => {
            collect_from_expression(owner, registry, out);
        }
        ExpressionKind::IndexedCall(call) => {
            for a in &call.indices {
                collect_from_expression(a, registry, out);
            }
        }
        ExpressionKind::VariadicAssignment { value, .. } => {
            collect_from_expression(value, registry, out);
        }
        ExpressionKind::Satisfies { subject, spec } => {
            collect_from_expression(subject, registry, out);
            collect_from_expression(spec, registry, out);
        }
        _ => {}
    }
}

fn collect_from_curly_args(
    args: &CurlyExpressionArgs,
    registry: &RenderRegistry,
    out: &mut Vec<FoundCommand>,
) {
    for e in &args.expressions {
        collect_from_expression(e, registry, out);
    }
}

fn collect_from_paren_args(
    args: &ParenExpressionArgs,
    registry: &RenderRegistry,
    out: &mut Vec<FoundCommand>,
) {
    for e in &args.expressions {
        collect_from_expression(e, registry, out);
    }
}

fn find_failing_command(
    formulation_text: &str,
    commands: &[FoundCommand],
    registry: &RenderRegistry,
) -> Option<FoundCommand> {
    // 1. Check if any command's own rendered LaTeX fails KaTeX
    for cmd in commands {
        if let Some(ref cmd_latex) = cmd.rendered_latex {
            if validate_latex(cmd_latex).is_err() {
                return Some(cmd.clone());
            }
        }
    }

    // 2. Try replacing each command with a dummy "x"
    for cmd in commands {
        if cmd.span.start < cmd.span.end && cmd.span.end <= formulation_text.len() {
            let replaced = format!(
                "{}x{}",
                &formulation_text[..cmd.span.start],
                &formulation_text[cmd.span.end..]
            );
            if let Some(rendered) = render_formulation_latex(&replaced, registry) {
                if validate_latex(&rendered).is_ok() {
                    return Some(cmd.clone());
                }
            }
        }
    }

    // 3. Fallback: if there is only 1 command in the formulation, it is the one!
    if commands.len() == 1 {
        return Some(commands[0].clone());
    }

    None
}

// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::view::build_render_registry;
    use crate::events::Audience;
    use crate::frontend::parse_source_file_from_source;

    #[test]
    fn validate_latex_accepts_valid_latex() {
        assert!(validate_latex(r"\frac{1}{2}").is_ok());
        assert!(validate_latex(r"x \in X").is_ok());
        assert!(validate_latex(r"\htmlData{mlg-ref=123}{\textrm{hello}}").is_ok());
        assert!(validate_latex(r"\htmlClass{mlg-title-written}{\textrm{hello}}").is_ok());
    }

    #[test]
    fn validate_latex_rejects_invalid_latex_and_cleans_message() {
        let err = validate_latex(r"\frac{1}").unwrap_err();
        assert!(err.starts_with("KaTeX parse error:"));
        assert!(err.contains("expected '}'"));

        let err2 = validate_latex(r"\undefinedmacrohere").unwrap_err();
        assert!(err2.starts_with("KaTeX parse error: Undefined control sequence:"));
    }

    #[test]
    fn latex_check_reports_warning_for_command_with_invalid_written_expansion() {
        let source = r#"[\thing]
Declares: value
Documented:
. called: "thing"

[\broken]
Defines: value
when: value is \thing
Documented:
. written: "\invalid{latex"

Theorem:
then:
. 1 + \broken + 2
"#;
        let mut event_log = EventLog::new();
        let path = PathBuf::from("test.mlg");
        let parsed_file = parse_source_file_from_source(path.clone(), source.to_string(), &mut event_log);
        let registry = build_render_registry(&[parsed_file.clone()]);

        let mut check_log = EventLog::new();
        let mut seen = HashSet::new();
        let locator = SourceLocator::new(&parsed_file.source);
        let mut proto_log = EventLog::new();
        let groups = ProtoParser::new(&parsed_file.source, &mut proto_log).parse();

        for group in &groups {
            check_group_latex(
                &parsed_file.path,
                group,
                &locator,
                &registry,
                &SourceFileFilter::All,
                &mut check_log,
                "test",
                &mut seen,
            );
        }

        let warnings: Vec<_> = check_log
            .events()
            .iter()
            .filter_map(|e| e.as_message())
            .filter(|m| m.audience == Audience::User && m.level == Level::Warning)
            .collect();

        assert!(!warnings.is_empty(), "expected LaTeX warnings");

        // There should be a warning specifically for `\broken` in `. 1 + \broken + 2`
        let cmd_warning = warnings
            .iter()
            .find(|m| {
                m.message.starts_with("For `\\broken`:")
                    && matches!(&m.location, Some(EventLocation::File { span: Some(span), .. }) if span.start.row == Some(13))
            })
            .expect("expected warning for `\\broken` in Theorem");

        assert!(cmd_warning.message.contains("KaTeX parse error:"));

        // Location of `. 1 + \broken + 2` is line 14 (0-based row 13), column of `\broken` is 6
        if let Some(EventLocation::File { span: Some(span), .. }) = &cmd_warning.location {
            assert_eq!(span.start.row, Some(13));
            assert_eq!(span.start.column, Some(6));
        } else {
            panic!("expected file location with span on warning");
        }
    }

    #[test]
    fn latex_check_reports_warning_for_text_entry_with_raw_latex_error() {
        let source = r#"Text:
. "This is broken math $ \frac{1} $ in text."
"#;
        let mut event_log = EventLog::new();
        let path = PathBuf::from("text_test.mlg");
        let parsed_file = parse_source_file_from_source(path.clone(), source.to_string(), &mut event_log);
        let registry = build_render_registry(&[parsed_file.clone()]);

        let mut check_log = EventLog::new();
        let mut seen = HashSet::new();
        let locator = SourceLocator::new(&parsed_file.source);
        let mut proto_log = EventLog::new();
        let groups = ProtoParser::new(&parsed_file.source, &mut proto_log).parse();

        for group in &groups {
            check_group_latex(
                &parsed_file.path,
                group,
                &locator,
                &registry,
                &SourceFileFilter::All,
                &mut check_log,
                "test",
                &mut seen,
            );
        }

        let warnings: Vec<_> = check_log
            .events()
            .iter()
            .filter_map(|e| e.as_message())
            .filter(|m| m.audience == Audience::User && m.level == Level::Warning)
            .collect();

        assert_eq!(warnings.len(), 1);
        let warning = warnings[0];
        assert!(warning.message.starts_with("KaTeX parse error:"));
        assert!(warning.message.contains("expected '}'"));

        // Line 2 (row 1), column of `$` is 23
        if let Some(EventLocation::File { span: Some(span), .. }) = &warning.location {
            assert_eq!(span.start.row, Some(1));
            assert_eq!(span.start.column, Some(23));
        } else {
            panic!("expected file location with span on warning");
        }
    }

    #[test]
    fn latex_check_reports_warning_for_text_entry_with_command_formulation_error() {
        let source = r#"[\thing]
Declares: value
Documented:
. called: "thing"

[\broken]
Defines: value
when: value is \thing
Documented:
. written: "\invalid{latex"

Text:
. "Reference {. \broken .} in prose."
"#;
        let mut event_log = EventLog::new();
        let path = PathBuf::from("text_cmd.mlg");
        let parsed_file = parse_source_file_from_source(path.clone(), source.to_string(), &mut event_log);
        let registry = build_render_registry(&[parsed_file.clone()]);

        let mut check_log = EventLog::new();
        let mut seen = HashSet::new();
        let locator = SourceLocator::new(&parsed_file.source);
        let mut proto_log = EventLog::new();
        let groups = ProtoParser::new(&parsed_file.source, &mut proto_log).parse();

        for group in &groups {
            check_group_latex(
                &parsed_file.path,
                group,
                &locator,
                &registry,
                &SourceFileFilter::All,
                &mut check_log,
                "test",
                &mut seen,
            );
        }

        let warnings: Vec<_> = check_log
            .events()
            .iter()
            .filter_map(|e| e.as_message())
            .filter(|m| m.audience == Audience::User && m.level == Level::Warning)
            .collect();

        let cmd_warning = warnings
            .iter()
            .find(|m| {
                m.message.starts_with("For `\\broken`:")
                    && matches!(&m.location, Some(EventLocation::File { span: Some(span), .. }) if span.start.row == Some(12))
            })
            .expect("expected warning for `\\broken` in text on line 13");

        assert!(cmd_warning.message.contains("KaTeX parse error:"));
        if let Some(EventLocation::File { span: Some(span), .. }) = &cmd_warning.location {
            assert_eq!(span.start.row, Some(12));
            assert_eq!(span.start.column, Some(16));
        }
    }
}
