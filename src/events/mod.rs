//! Event reporting infrastructure shared by parsers, checks, and CLI commands.
//!
//! Events carry audience, level, origin, and optional location metadata.  The
//! command layer wires an `EventConsoleWriter` listener to an `EventLog` so each
//! operation can emit diagnostics without depending directly on stdout/stderr.

mod event;
mod event_console_writer;
mod event_filter;
mod event_location;
mod event_log;
mod event_position;
mod event_span;
mod level;
mod marker_event;
mod marker_id;
mod marker_range;
mod message_event;

pub use event::{Audience, Event};
pub use event_console_writer::EventConsoleWriter;
pub use event_filter::{ColorMode, EventFilter};
pub use event_location::EventLocation;
pub use event_log::{EventLog, EventLogListener};
pub use event_position::EventPosition;
pub use event_span::EventSpan;
pub use level::Level;
pub use marker_event::{MarkerEvent, MarkerPhase};
pub use marker_id::MarkerId;
pub use marker_range::MarkerRange;
pub use message_event::MessageEvent;
