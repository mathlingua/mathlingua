//! Event reporting infrastructure shared by parsers, checks, and CLI commands.
//!
//! Events carry audience, level, origin, and optional location metadata.  The
//! command layer wires an `EventConsoleWriter` listener to an `EventLog` so each
//! operation can emit diagnostics without depending directly on stdout/stderr.

mod audience;
mod console_writer;
mod event;
mod filter;
mod level;
mod location;
mod log;
mod marker_event;
mod marker_id;
mod marker_range;
mod message_event;

/// Event audience classification.
pub use audience::Audience;
/// Event listener that renders events to stdout/stderr.
pub use console_writer::EventConsoleWriter;
/// Top-level event enum.
pub use event::Event;
/// Event filtering and color configuration.
pub use filter::{ColorMode, EventFilter};
/// Event severity/purpose.
pub use level::Level;
/// Source location model used by diagnostics.
pub use location::{EventLocation, EventPosition, EventSpan};
/// Append-only event log and listener trait.
pub use log::{EventLog, EventLogListener};
pub use marker_event::{MarkerEvent, MarkerPhase};
pub use marker_id::MarkerId;
/// Marker event types for slicing/log instrumentation.
pub use marker_range::MarkerRange;
/// User/system message event payload.
pub use message_event::MessageEvent;
