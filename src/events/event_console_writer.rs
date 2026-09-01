use super::event_filter::{ColorMode, EventFilter};
use super::event_log::EventLogListener;
use super::{
    Audience, Event, EventLocation, EventSpan, Level, MarkerEvent, MarkerPhase, MessageEvent,
    MessageStatus,
};
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

pub struct EventConsoleWriter {
    filter: EventFilter,
    base_path: Option<PathBuf>,
    color_mode: ColorMode,
    status_active: bool,
}

impl Default for EventConsoleWriter {
    fn default() -> Self {
        Self {
            filter: EventFilter::default(),
            base_path: None,
            color_mode: ColorMode::Auto,
            status_active: false,
        }
    }
}

impl EventConsoleWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_filter(mut self, filter: EventFilter) -> Self {
        self.filter = filter;
        self
    }

    pub fn with_base_path(mut self, base_path: impl Into<PathBuf>) -> Self {
        self.base_path = Some(base_path.into());
        self
    }

    pub fn with_color_mode(mut self, color_mode: ColorMode) -> Self {
        self.color_mode = color_mode;
        self
    }

    /// The line `event` would be printed as, without choosing a stream for it.
    /// `None` when the filter excludes it.
    ///
    /// This is for callers that route console-shaped diagnostics somewhere other
    /// than the console — `mlg extract` prints them into its stdout payload — so
    /// that the wording, paths, and line/column format stay identical to what
    /// `mlg check` shows.
    pub fn render_to_string(&self, event: &Event) -> Option<String> {
        self.render(event).map(|rendered| rendered.text)
    }

    fn render(&self, event: &Event) -> Option<RenderedEvent> {
        if !self.filter.matches(event) {
            return None;
        }

        match event {
            Event::Message(message) => Some(self.render_message(message)),
            Event::Marker(marker) => Some(self.render_marker(marker)),
        }
    }

    fn render_message(&self, event: &MessageEvent) -> RenderedEvent {
        let use_color = self.should_use_color();
        let destination = if event.audience == Audience::User && event.level == Level::Log {
            ConsoleDestination::Stdout
        } else {
            ConsoleDestination::Stderr
        };

        let text = if event.audience == Audience::User && event.level == Level::Log {
            event.message.clone()
        } else {
            let prefix = self.render_prefix(event, use_color);
            match self.render_location(event.location.as_ref()) {
                Some(location) => format!("{location}: {prefix}: {}", event.message),
                None => format!("{prefix}: {}", event.message),
            }
        };

        RenderedEvent {
            text,
            destination,
            status: event.status,
        }
    }

    fn render_marker(&self, marker: &MarkerEvent) -> RenderedEvent {
        let phase = match marker.phase {
            MarkerPhase::Begin => "begin",
            MarkerPhase::End => "end",
        };
        let origin = marker
            .origin
            .as_deref()
            .map(|origin| format!(" [{origin}]"))
            .unwrap_or_default();

        RenderedEvent {
            text: format!("marker {phase}{origin}: {} ({})", marker.label, marker.id),
            destination: ConsoleDestination::Stdout,
            status: None,
        }
    }

    fn render_prefix(&self, event: &MessageEvent, use_color: bool) -> String {
        let level = match event.level {
            Level::Log => style_label("log", Style::Blue, use_color),
            Level::Warning => style_label("warning", Style::Yellow, use_color),
            Level::Error => style_label("error", Style::Red, use_color),
            Level::Debug => style_label("debug", Style::Magenta, use_color),
        };

        match event.audience {
            Audience::User => level,
            Audience::System => {
                let origin = event
                    .origin
                    .as_deref()
                    .map(|origin| format!(" [{origin}]"))
                    .unwrap_or_default();
                format!("system {level}{origin}")
            }
        }
    }

    fn render_location(&self, location: Option<&EventLocation>) -> Option<String> {
        match location? {
            EventLocation::File { path, span } => Some(match span {
                Some(span) => format!("{}{}", self.display_path(path), render_file_span(span)),
                None => self.display_path(path),
            }),
            EventLocation::InMemory { name, span } => match (name, span) {
                (Some(name), Some(span)) => Some(format!("{name}: {}", render_memory_span(span))),
                (Some(name), None) => Some(name.clone()),
                (None, Some(span)) => Some(render_memory_span(span)),
                (None, None) => None,
            },
        }
    }

    fn display_path(&self, path: &Path) -> String {
        self.base_path
            .as_deref()
            .and_then(|base| path.strip_prefix(base).ok())
            .map(display_relative_path)
            .unwrap_or_else(|| path.display().to_string())
    }

    fn should_use_color(&self) -> bool {
        match self.color_mode {
            ColorMode::Auto => std::io::stderr().is_terminal(),
            ColorMode::Always => true,
            ColorMode::Never => false,
        }
    }
}

impl EventLogListener for EventConsoleWriter {
    fn on_event(&mut self, event: &Event) {
        let Some(rendered) = self.render(event) else {
            return;
        };

        let stdout_is_terminal = std::io::stdout().is_terminal();
        if stdout_is_terminal {
            match rendered.status {
                Some(MessageStatus::Started) => {
                    if self.status_active {
                        let _ = finish_status_line(std::io::stdout().lock());
                    }
                    let icon = style_label("◌", Style::Blue, self.should_use_color());
                    let _ = write_status(std::io::stdout().lock(), &icon, &rendered.text, false);
                    self.status_active = true;
                    return;
                }
                Some(MessageStatus::Finished) => {
                    let icon = style_label("✓", Style::Green, self.should_use_color());
                    let _ = write_status(std::io::stdout().lock(), &icon, &rendered.text, true);
                    self.status_active = false;
                    return;
                }
                None if self.status_active => {
                    let _ = finish_status_line(std::io::stdout().lock());
                    self.status_active = false;
                }
                None => {}
            }
        }

        let _ = match rendered.destination {
            ConsoleDestination::Stdout => write_line(std::io::stdout().lock(), &rendered.text),
            ConsoleDestination::Stderr => write_line(std::io::stderr().lock(), &rendered.text),
        };
    }

    fn clear_output(&mut self) {
        if !std::io::stdout().is_terminal() {
            return;
        }

        let _ = clear_terminal(std::io::stdout().lock());
        self.status_active = false;
    }
}

impl Drop for EventConsoleWriter {
    fn drop(&mut self) {
        if self.status_active && std::io::stdout().is_terminal() {
            let _ = finish_status_line(std::io::stdout().lock());
        }
    }
}

struct RenderedEvent {
    text: String,
    destination: ConsoleDestination,
    status: Option<MessageStatus>,
}

enum ConsoleDestination {
    Stdout,
    Stderr,
}

#[derive(Clone, Copy)]
enum Style {
    Red,
    Yellow,
    Blue,
    Green,
    Magenta,
}

fn render_file_span(span: &EventSpan) -> String {
    match (
        &span.start.row,
        &span.start.column,
        &span.start.offset,
        &span.end,
    ) {
        (Some(row), Some(column), _, Some(end)) if end.row == Some(*row) => match end.column {
            Some(end_column) => format!(":{}:{}-{}", row + 1, column + 1, end_column + 1),
            None => format!(":{}:{}", row + 1, column + 1),
        },
        (Some(row), Some(column), _, Some(end)) => match (end.row, end.column) {
            (Some(end_row), Some(end_column)) => {
                format!(
                    ":{}:{}-{}:{}",
                    row + 1,
                    column + 1,
                    end_row + 1,
                    end_column + 1
                )
            }
            _ => format!(":{}:{}", row + 1, column + 1),
        },
        (Some(row), Some(column), _, None) => format!(":{}:{}", row + 1, column + 1),
        (Some(row), None, _, Some(end)) if end.row != Some(*row) => match end.row {
            Some(end_row) => format!(":{}-{}", row + 1, end_row + 1),
            None => format!(":{}", row + 1),
        },
        (Some(row), None, _, _) => format!(":{}", row + 1),
        (None, None, Some(offset), Some(end)) if end.offset.is_some() => {
            format!("@{}-{}", offset, end.offset.unwrap())
        }
        (None, None, Some(offset), _) => format!("@{offset}"),
        _ => String::new(),
    }
}

fn render_memory_span(span: &EventSpan) -> String {
    match (
        &span.start.row,
        &span.start.column,
        &span.start.offset,
        &span.end,
    ) {
        (Some(row), Some(column), _, Some(end)) => match (end.row, end.column) {
            (Some(end_row), Some(end_column)) => format!(
                "line {}, column {} to line {}, column {}",
                row + 1,
                column + 1,
                end_row + 1,
                end_column + 1
            ),
            _ => format!("line {}, column {}", row + 1, column + 1),
        },
        (Some(row), Some(column), _, None) => format!("line {}, column {}", row + 1, column + 1),
        (Some(row), None, _, Some(end)) => match end.row {
            Some(end_row) => format!("line {} to line {}", row + 1, end_row + 1),
            None => format!("line {}", row + 1),
        },
        (Some(row), None, _, None) => format!("line {}", row + 1),
        (None, None, Some(offset), Some(end)) => match end.offset {
            Some(end_offset) => format!("offset {offset} to offset {end_offset}"),
            None => format!("offset {offset}"),
        },
        (None, None, Some(offset), None) => format!("offset {offset}"),
        _ => "location".to_string(),
    }
}

fn style_label(text: &str, style: Style, use_color: bool) -> String {
    if !use_color {
        return text.to_string();
    }

    let code = match style {
        Style::Red => "1;31",
        Style::Yellow => "1;33",
        Style::Blue => "1;34",
        Style::Green => "1;32",
        Style::Magenta => "1;35",
    };

    format!("\x1b[{code}m{text}\x1b[0m")
}

fn write_line(mut writer: impl Write, message: &str) -> io::Result<()> {
    writer.write_all(message.as_bytes())?;
    writer.write_all(b"\n")?;
    writer.flush()
}

fn write_status(mut writer: impl Write, icon: &str, message: &str, finish: bool) -> io::Result<()> {
    writer.write_all(b"\r\x1b[2K")?;
    write!(writer, "{icon} {message}")?;
    if finish {
        writer.write_all(b"\n")?;
    }
    writer.flush()
}

fn finish_status_line(mut writer: impl Write) -> io::Result<()> {
    writer.write_all(b"\n")?;
    writer.flush()
}

fn clear_terminal(mut writer: impl Write) -> io::Result<()> {
    writer.write_all(b"\x1b[2J\x1b[H")?;
    writer.flush()
}

fn display_relative_path(path: &Path) -> String {
    let relative = path.strip_prefix("content").unwrap_or(path);
    if relative.as_os_str().is_empty() {
        ".".to_string()
    } else {
        relative.display().to_string()
    }
}

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::{ColorMode, EventConsoleWriter, EventFilter, clear_terminal, write_status};
    use crate::events::{Audience, Event, EventLocation, EventSpan, Level, MessageStatus};
    use std::path::Path;

    #[test]
    fn renders_user_logs_as_plain_messages() {
        let writer = EventConsoleWriter::new().with_color_mode(ColorMode::Never);
        let rendered = writer.render(&Event::user_log("Checked 2 files")).unwrap();

        assert_eq!(rendered.text, "Checked 2 files");
    }

    #[test]
    fn renders_user_errors_relative_to_the_base_path() {
        let writer = EventConsoleWriter::new()
            .with_base_path(Path::new("/repo"))
            .with_color_mode(ColorMode::Never);
        let rendered = writer
            .render(
                &Event::message(
                    "Unexpected header: [duplicate]",
                    Level::Error,
                    Audience::User,
                    Some(EventLocation::file(
                        "/repo/content/sets/example.mlg",
                        Some(EventSpan::row(3)),
                    )),
                )
                .with_origin("structural_parser"),
            )
            .unwrap();

        assert_eq!(
            rendered.text,
            "sets/example.mlg:4: error: Unexpected header: [duplicate]"
        );
    }

    #[test]
    fn renders_system_events_with_origin_information() {
        let writer = EventConsoleWriter::new()
            .with_filter(EventFilter::new().with_audiences(vec![Audience::System]))
            .with_color_mode(ColorMode::Never);
        let rendered = writer
            .render(&Event::system_debug("Parsing file").with_origin("mlg_check"))
            .unwrap();

        assert_eq!(rendered.text, "system debug [mlg_check]: Parsing file");
    }

    #[test]
    fn filters_out_non_matching_audiences() {
        let writer = EventConsoleWriter::new()
            .with_filter(EventFilter::new().with_audiences(vec![Audience::System]));

        assert!(writer.render(&Event::user_log("Checked 1 file")).is_none());
    }

    #[test]
    fn preserves_status_metadata_for_console_rendering() {
        let writer = EventConsoleWriter::new().with_color_mode(ColorMode::Never);
        let rendered = writer
            .render(&Event::user_status(
                "Starting viewer",
                MessageStatus::Started,
            ))
            .unwrap();

        assert_eq!(rendered.text, "Starting viewer");
        assert_eq!(rendered.status, Some(MessageStatus::Started));
    }

    #[test]
    fn writes_a_status_update_on_one_terminal_line() {
        let mut output = Vec::new();

        write_status(&mut output, "✓", "Viewer ready", true).unwrap();

        assert_eq!(output, b"\r\x1b[2K\xe2\x9c\x93 Viewer ready\n");
    }

    #[test]
    fn clears_the_terminal_and_returns_to_the_top_left() {
        let mut output = Vec::new();

        clear_terminal(&mut output).unwrap();

        assert_eq!(output, b"\x1b[2J\x1b[H");
    }
}
