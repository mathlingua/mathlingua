use crate::events::{Event, EventLocation, EventLog, EventLogListener, EventPosition, Level};
use crate::frontend::formulation::{
    ParseError, parse_command_header, parse_expression, parse_form_or_declaration,
    parse_is_via_statement, parse_refined_declaration_statement,
};
use crate::frontend::structural::parse_document;
use crossterm::event::{
    self, Event as TerminalEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Tabs, Wrap};
use ratatui::{Frame, Terminal};
use std::io::{self, Stdout};
use std::time::Duration;

const ORIGIN: &str = "mlg_debug";

pub struct DebugResult {
    pub event_log: EventLog,
    pub successful: bool,
}

pub fn debug(listener: Option<Box<dyn EventLogListener>>) -> DebugResult {
    let mut event_log = EventLog::new();
    if let Some(listener) = listener {
        event_log.add_boxed_listener(listener);
    }

    let successful = match run_debug_tui() {
        Ok(()) => true,
        Err(error) => {
            event_log.user_error(Some(ORIGIN), format!("Debug TUI failed: {error}"));
            false
        }
    };

    DebugResult {
        event_log,
        successful,
    }
}

fn run_debug_tui() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    if let Err(error) = execute!(stdout, EnterAlternateScreen) {
        let _ = disable_raw_mode();
        return Err(error);
    }

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = match Terminal::new(backend) {
        Ok(terminal) => terminal,
        Err(error) => {
            let _ = disable_raw_mode();
            let mut stdout = io::stdout();
            let _ = execute!(stdout, LeaveAlternateScreen);
            return Err(error);
        }
    };
    let result = run_app(&mut terminal);

    let raw_mode_result = disable_raw_mode();
    let leave_result = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let cursor_result = terminal.show_cursor();

    result?;
    raw_mode_result?;
    leave_result?;
    cursor_result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    let mut app = DebugApp::default();

    loop {
        terminal.draw(|frame| draw(frame, &mut app))?;

        if !event::poll(Duration::from_millis(100))? {
            continue;
        }

        let TerminalEvent::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        if app.handle_key(key) {
            break;
        }
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum DebugMode {
    #[default]
    Formulation,
    Structural,
    CommandHeader,
}

impl DebugMode {
    const ALL: [Self; 3] = [Self::Formulation, Self::Structural, Self::CommandHeader];

    fn title(self) -> &'static str {
        match self {
            Self::Formulation => "Formulation",
            Self::Structural => "Structural",
            Self::CommandHeader => "Command Header",
        }
    }

    fn next(self) -> Self {
        let index = Self::ALL
            .iter()
            .position(|mode| *mode == self)
            .unwrap_or_default();
        Self::ALL[(index + 1) % Self::ALL.len()]
    }

    fn previous(self) -> Self {
        let index = Self::ALL
            .iter()
            .position(|mode| *mode == self)
            .unwrap_or_default();
        Self::ALL[(index + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    fn tab_index(self) -> usize {
        Self::ALL
            .iter()
            .position(|mode| *mode == self)
            .unwrap_or_default()
    }
}

#[derive(Debug, Default)]
struct DebugApp {
    mode: DebugMode,
    input: TextBuffer,
    output_scroll: u16,
    last_input_area: Rect,
}

impl DebugApp {
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('c') | KeyCode::Char('q') => return true,
                KeyCode::Char('1') => self.set_mode(DebugMode::Formulation),
                KeyCode::Char('2') => self.set_mode(DebugMode::Structural),
                KeyCode::Char('3') => self.set_mode(DebugMode::CommandHeader),
                _ => {}
            }
            return false;
        }

        match key.code {
            KeyCode::Esc => return true,
            KeyCode::Tab => self.set_mode(self.mode.next()),
            KeyCode::BackTab => self.set_mode(self.mode.previous()),
            KeyCode::F(1) => self.set_mode(DebugMode::Formulation),
            KeyCode::F(2) => self.set_mode(DebugMode::Structural),
            KeyCode::F(3) => self.set_mode(DebugMode::CommandHeader),
            KeyCode::PageUp => self.output_scroll = self.output_scroll.saturating_sub(8),
            KeyCode::PageDown => self.output_scroll = self.output_scroll.saturating_add(8),
            KeyCode::Up => self.input.move_up(self.input_height()),
            KeyCode::Down => self.input.move_down(self.input_height()),
            KeyCode::Left => self.input.move_left(self.input_height()),
            KeyCode::Right => self.input.move_right(self.input_height()),
            KeyCode::Home => self.input.move_home(self.input_height()),
            KeyCode::End => self.input.move_end(self.input_height()),
            KeyCode::Backspace => self.input.backspace(self.input_height()),
            KeyCode::Delete => self.input.delete(self.input_height()),
            KeyCode::Enter => self.input.insert_newline(self.input_height()),
            KeyCode::Char(ch) => self.input.insert_char(ch, self.input_height()),
            _ => {}
        }

        false
    }

    fn set_mode(&mut self, mode: DebugMode) {
        self.mode = mode;
        self.output_scroll = 0;
    }

    fn input_height(&self) -> usize {
        self.last_input_area.height.saturating_sub(2).max(1) as usize
    }
}

#[derive(Debug)]
struct TextBuffer {
    lines: Vec<String>,
    cursor_row: usize,
    cursor_col: usize,
    scroll_row: usize,
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            scroll_row: 0,
        }
    }
}

impl TextBuffer {
    fn text(&self) -> String {
        self.lines.join("\n")
    }

    fn visible_text(&self, height: usize) -> String {
        self.lines
            .iter()
            .skip(self.scroll_row)
            .take(height.max(1))
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn insert_char(&mut self, ch: char, height: usize) {
        if ch == '\n' || ch == '\r' {
            self.insert_newline(height);
            return;
        }

        let index = byte_index_for_col(&self.lines[self.cursor_row], self.cursor_col);
        self.lines[self.cursor_row].insert(index, ch);
        self.cursor_col += 1;
        self.ensure_cursor_visible(height);
    }

    fn insert_newline(&mut self, height: usize) {
        let index = byte_index_for_col(&self.lines[self.cursor_row], self.cursor_col);
        let tail = self.lines[self.cursor_row].split_off(index);
        self.cursor_row += 1;
        self.cursor_col = 0;
        self.lines.insert(self.cursor_row, tail);
        self.ensure_cursor_visible(height);
    }

    fn backspace(&mut self, height: usize) {
        if self.cursor_col > 0 {
            let index = byte_index_for_col(&self.lines[self.cursor_row], self.cursor_col);
            let previous = byte_index_for_col(&self.lines[self.cursor_row], self.cursor_col - 1);
            self.lines[self.cursor_row].replace_range(previous..index, "");
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            let current = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = char_len(&self.lines[self.cursor_row]);
            self.lines[self.cursor_row].push_str(&current);
        }

        self.ensure_cursor_visible(height);
    }

    fn delete(&mut self, height: usize) {
        let line_len = char_len(&self.lines[self.cursor_row]);
        if self.cursor_col < line_len {
            let start = byte_index_for_col(&self.lines[self.cursor_row], self.cursor_col);
            let end = byte_index_for_col(&self.lines[self.cursor_row], self.cursor_col + 1);
            self.lines[self.cursor_row].replace_range(start..end, "");
        } else if self.cursor_row + 1 < self.lines.len() {
            let next = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next);
        }

        self.ensure_cursor_visible(height);
    }

    fn move_left(&mut self, height: usize) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = char_len(&self.lines[self.cursor_row]);
        }

        self.ensure_cursor_visible(height);
    }

    fn move_right(&mut self, height: usize) {
        if self.cursor_col < char_len(&self.lines[self.cursor_row]) {
            self.cursor_col += 1;
        } else if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }

        self.ensure_cursor_visible(height);
    }

    fn move_up(&mut self, height: usize) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.cursor_col.min(char_len(&self.lines[self.cursor_row]));
        }

        self.ensure_cursor_visible(height);
    }

    fn move_down(&mut self, height: usize) {
        if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = self.cursor_col.min(char_len(&self.lines[self.cursor_row]));
        }

        self.ensure_cursor_visible(height);
    }

    fn move_home(&mut self, height: usize) {
        self.cursor_col = 0;
        self.ensure_cursor_visible(height);
    }

    fn move_end(&mut self, height: usize) {
        self.cursor_col = char_len(&self.lines[self.cursor_row]);
        self.ensure_cursor_visible(height);
    }

    fn ensure_cursor_visible(&mut self, height: usize) {
        let height = height.max(1);
        if self.cursor_row < self.scroll_row {
            self.scroll_row = self.cursor_row;
        } else if self.cursor_row >= self.scroll_row + height {
            self.scroll_row = self.cursor_row + 1 - height;
        }
    }

    fn cursor_position(&self, area: Rect) -> Option<Position> {
        let visible_row = self.cursor_row.checked_sub(self.scroll_row)?;
        let content_height = area.height.saturating_sub(2) as usize;
        let content_width = area.width.saturating_sub(2) as usize;
        if visible_row >= content_height {
            return None;
        }

        Some(Position::new(
            area.x + 1 + self.cursor_col.min(content_width) as u16,
            area.y + 1 + visible_row as u16,
        ))
    }
}

fn draw(frame: &mut Frame<'_>, app: &mut DebugApp) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(frame.area());

    draw_tabs(frame, layout[0], app.mode);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
        .split(layout[1]);

    app.last_input_area = body[0];
    app.input.ensure_cursor_visible(app.input_height());
    draw_input(frame, body[0], app);
    draw_output(frame, body[1], app);
    draw_help(frame, layout[2]);

    if let Some(position) = app.input.cursor_position(body[0]) {
        frame.set_cursor_position(position);
    }
}

fn draw_tabs(frame: &mut Frame<'_>, area: Rect, mode: DebugMode) {
    let titles = DebugMode::ALL
        .iter()
        .map(|mode| Line::from(Span::raw(mode.title())));
    let tabs = Tabs::new(titles)
        .select(mode.tab_index())
        .block(Block::default().borders(Borders::BOTTOM))
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, area);
}

fn draw_input(frame: &mut Frame<'_>, area: Rect, app: &DebugApp) {
    let block = Block::default()
        .title(format!("Input ({})", app.mode.title()))
        .borders(Borders::ALL);
    let paragraph = Paragraph::new(app.input.visible_text(app.input_height()))
        .block(block)
        .style(Style::default().fg(Color::White));

    frame.render_widget(paragraph, area);
}

fn draw_output(frame: &mut Frame<'_>, area: Rect, app: &DebugApp) {
    let text = parse_text(app.mode, &app.input.text());
    let block = Block::default().title("Parse result").borders(Borders::ALL);
    let paragraph = Paragraph::new(text)
        .block(block)
        .style(Style::default().fg(Color::White))
        .scroll((app.output_scroll, 0))
        .wrap(Wrap { trim: false });

    frame.render_widget(Clear, area);
    frame.render_widget(paragraph, area);
}

fn draw_help(frame: &mut Frame<'_>, area: Rect) {
    let help = Paragraph::new(
        "Tab/Shift-Tab or F1-F3: mode    PageUp/PageDown: result scroll    Esc/Ctrl-C/Ctrl-Q: quit",
    )
    .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(help, area);
}

fn parse_text(mode: DebugMode, input: &str) -> String {
    if input.trim().is_empty() {
        return format!(
            "{} mode\n\nType MathLingua text in the left panel to parse it.",
            mode.title()
        );
    }

    match mode {
        DebugMode::Formulation => parse_formulation_text(input),
        DebugMode::Structural => parse_structural_text(input),
        DebugMode::CommandHeader => parse_command_header_text(input),
    }
}

fn parse_formulation_text(input: &str) -> String {
    let attempts = [
        FormulationAttempt::new("expression", parse_expression(input)),
        FormulationAttempt::new(
            "refined declaration statement",
            parse_refined_declaration_statement(input),
        ),
        FormulationAttempt::new("form or declaration", parse_form_or_declaration(input)),
        FormulationAttempt::new("is-via statement", parse_is_via_statement(input)),
    ];

    let Some(success) = attempts.iter().find_map(FormulationAttempt::success) else {
        let errors = attempts
            .iter()
            .map(FormulationAttempt::error_summary)
            .collect::<Vec<_>>()
            .join("\n\n");
        return format!("Parse errors\n\n{errors}");
    };

    let also_parses_as = attempts
        .iter()
        .filter(|attempt| attempt.name != success.name && attempt.parsed.is_ok())
        .map(|attempt| format!("- {}", attempt.name))
        .collect::<Vec<_>>();

    let mut output = format!("Parsed as {}\n\n{}", success.name, success.tree);
    if !also_parses_as.is_empty() {
        output.push_str("\n\nAlso parses as:\n");
        output.push_str(&also_parses_as.join("\n"));
    }

    output
}

fn parse_structural_text(input: &str) -> String {
    let mut event_log = EventLog::new();
    let document = parse_document(input, &mut event_log);
    let diagnostics = format_events(event_log.events());
    let diagnostics = if diagnostics.is_empty() {
        "No parse errors.".to_string()
    } else {
        diagnostics
    };

    format!("Diagnostics\n\n{diagnostics}\n\nParse tree\n\n{document:#?}")
}

fn parse_command_header_text(input: &str) -> String {
    match parse_command_header(input) {
        Ok(header) => format!("Parsed command header\n\n{header:#?}"),
        Err(error) => format!("Parse error\n\n{error}\n\nDebug\n\n{error:#?}"),
    }
}

struct FormulationAttempt {
    name: &'static str,
    parsed: Result<String, ParseError>,
}

impl FormulationAttempt {
    fn new<T: std::fmt::Debug>(name: &'static str, result: Result<T, ParseError>) -> Self {
        Self {
            name,
            parsed: result.map(|tree| format!("{tree:#?}")),
        }
    }

    fn success(&self) -> Option<FormulationSuccess<'_>> {
        self.parsed.as_ref().ok().map(|tree| FormulationSuccess {
            name: self.name,
            tree,
        })
    }

    fn error_summary(&self) -> String {
        match &self.parsed {
            Ok(_) => format!("{}: ok", self.name),
            Err(error) => format!("{}:\n  {}", self.name, indent_lines(&error.to_string(), 2)),
        }
    }
}

struct FormulationSuccess<'a> {
    name: &'static str,
    tree: &'a str,
}

fn format_events(events: &[Event]) -> String {
    events
        .iter()
        .filter_map(Event::as_message)
        .map(|message| {
            let location = message
                .location
                .as_ref()
                .map(format_location)
                .unwrap_or_default();
            let origin = message
                .origin
                .as_ref()
                .map(|origin| format!(" [{origin}]"))
                .unwrap_or_default();

            format!(
                "{}{}{}: {}",
                format_level(message.level),
                location,
                origin,
                message.message
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_level(level: Level) -> &'static str {
    match level {
        Level::Log => "log",
        Level::Warning => "warning",
        Level::Error => "error",
        Level::Debug => "debug",
    }
}

fn format_location(location: &EventLocation) -> String {
    match location {
        EventLocation::InMemory { span, .. } => span
            .as_ref()
            .map(format_span)
            .map(|span| format!(" {span}"))
            .unwrap_or_default(),
        EventLocation::File { path, span } => {
            let span = span
                .as_ref()
                .map(format_span)
                .map(|span| format!(" {span}"))
                .unwrap_or_default();
            format!(" {}{}", path.display(), span)
        }
    }
}

fn format_span(span: &crate::events::EventSpan) -> String {
    format_position(&span.start)
}

fn format_position(position: &EventPosition) -> String {
    match (position.row, position.column, position.offset) {
        (Some(row), Some(column), _) => format!("{row}:{column}"),
        (Some(row), None, _) => format!("{row}"),
        (None, None, Some(offset)) => format!("offset {offset}"),
        _ => String::new(),
    }
}

fn indent_lines(text: &str, spaces: usize) -> String {
    let padding = " ".repeat(spaces);
    text.lines()
        .map(|line| format!("{padding}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn char_len(text: &str) -> usize {
    text.chars().count()
}

fn byte_index_for_col(text: &str, col: usize) -> usize {
    text.char_indices()
        .nth(col)
        .map(|(index, _)| index)
        .unwrap_or(text.len())
}

#[cfg(test)]
mod tests {
    use super::{DebugMode, parse_text};

    #[test]
    fn formulation_mode_shows_a_parse_tree() {
        let output = parse_text(DebugMode::Formulation, "x + y");

        assert!(output.contains("Parsed as expression"));
        assert!(output.contains("Expression {\n"));
        assert!(!output.contains("\\n"));
    }

    #[test]
    fn command_header_mode_shows_errors() {
        let output = parse_text(DebugMode::CommandHeader, "x + y");

        assert!(output.contains("Parse error"));
        assert!(output.contains("command header must start"));
    }

    #[test]
    fn structural_mode_shows_document_tree() {
        let output = parse_text(DebugMode::Structural, "Title: \"Intro\"");

        assert!(output.contains("No parse errors."));
        assert!(output.contains("Parse tree"));
    }
}
