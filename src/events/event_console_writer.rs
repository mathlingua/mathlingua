use super::event_filter::EventFilter;
use super::event_log::EventLogListener;
use super::{
    Audience, Event, EventLocation, EventSpan, Level, MarkerEvent, MarkerPhase, MessageEvent,
};
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorMode {
    #[default]
    Auto,
    Always,
    Never,
}

pub struct EventConsoleWriter {
    filter: EventFilter,
    base_path: Option<PathBuf>,
    color_mode: ColorMode,
}

impl Default for EventConsoleWriter {
    fn default() -> Self {
        Self {
            filter: EventFilter::default(),
            base_path: None,
            color_mode: ColorMode::Auto,
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

        RenderedEvent { text, destination }
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
        }
    }

    fn render_prefix(&self, event: &MessageEvent, use_color: bool) -> String {
        let level = match event.level {
            Level::Log => style_label("LOG", Style::Blue, use_color),
            Level::Warning => style_label("WARNING", Style::Yellow, use_color),
            Level::Error => style_label("ERROR", Style::Red, use_color),
            Level::Debug => style_label("DEBUG", Style::Magenta, use_color),
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
            .map(|relative| {
                if relative.as_os_str().is_empty() {
                    ".".to_string()
                } else {
                    relative.display().to_string()
                }
            })
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

        let _ = match rendered.destination {
            ConsoleDestination::Stdout => write_line(std::io::stdout().lock(), &rendered.text),
            ConsoleDestination::Stderr => write_line(std::io::stderr().lock(), &rendered.text),
        };
    }
}

struct RenderedEvent {
    text: String,
    destination: ConsoleDestination,
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
        Style::Magenta => "1;35",
    };

    format!("\x1b[{code}m{text}\x1b[0m")
}

fn write_line(mut writer: impl Write, message: &str) -> io::Result<()> {
    writer.write_all(message.as_bytes())?;
    writer.write_all(b"\n")?;
    writer.flush()
}

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use crate::events::{
        Audience, ColorMode, Event, EventConsoleWriter, EventFilter, EventLocation, EventSpan,
        Level,
    };
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
                        "/repo/content/example.mlg",
                        Some(EventSpan::row(3)),
                    )),
                )
                .with_origin("structural_parser"),
            )
            .unwrap();

        assert_eq!(
            rendered.text,
            "content/example.mlg:4: ERROR: Unexpected header: [duplicate]"
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

        assert_eq!(rendered.text, "system DEBUG [mlg_check]: Parsing file");
    }

    #[test]
    fn filters_out_non_matching_audiences() {
        let writer = EventConsoleWriter::new()
            .with_filter(EventFilter::new().with_audiences(vec![Audience::System]));

        assert!(writer.render(&Event::user_log("Checked 1 file")).is_none());
    }
}
