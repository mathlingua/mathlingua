use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Monotonic counter mixed into marker ids to avoid collisions inside one process.
static NEXT_MARKER_ID: AtomicU64 = AtomicU64::new(0);

/// Identifier shared by the begin and end marker events of a measured range.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MarkerId(String);

impl MarkerId {
    /// Creates a new UUID-shaped marker id.
    ///
    /// This is not intended to be cryptographically random; it combines current
    /// time with a process-local counter and sets UUID version/variant bits so it
    /// is easy to read and group in logs.
    pub fn new() -> Self {
        let mut bytes = [0u8; 16];
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let counter = NEXT_MARKER_ID.fetch_add(1, Ordering::Relaxed) as u128;
        let raw = now ^ counter.rotate_left(17);

        bytes.copy_from_slice(&raw.to_le_bytes());
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;

        Self(format_uuid(bytes))
    }

    /// Returns the id as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for MarkerId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for MarkerId {
    /// Writes the UUID-shaped marker id.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Phase of an instrumentation marker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkerPhase {
    /// Beginning of a marked range.
    Begin,
    /// End of a marked range.
    End,
}

/// Event emitted when a marked range begins or ends.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkerEvent {
    /// Id shared between matching begin and end marker events.
    pub id: MarkerId,
    /// Human-readable label for the measured range.
    pub label: String,
    /// Whether this event begins or ends the range.
    pub phase: MarkerPhase,
    /// Optional subsystem that emitted the marker.
    pub origin: Option<String>,
}

impl MarkerEvent {
    /// Creates a marker event from its components.
    pub fn new(
        id: MarkerId,
        label: impl Into<String>,
        phase: MarkerPhase,
        origin: Option<String>,
    ) -> Self {
        Self {
            id,
            label: label.into(),
            phase,
            origin,
        }
    }

    /// Returns a marker event with the given optional origin.
    pub fn with_origin_option(mut self, origin: Option<&str>) -> Self {
        self.origin = origin.map(str::to_owned);
        self
    }
}

/// Pair of begin and end marker events bounding a range in an event log.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkerRange {
    /// Begin marker event.
    pub begin: MarkerEvent,
    /// End marker event.
    pub end: MarkerEvent,
}

impl MarkerRange {
    /// Creates a marker range from matching begin and end events.
    pub fn new(begin: MarkerEvent, end: MarkerEvent) -> Self {
        Self { begin, end }
    }
}

/// Formats raw bytes as a UUID-shaped string.
fn format_uuid(bytes: [u8; 16]) -> String {
    let hex = bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>();

    format!(
        "{}{}{}{}-{}{}-{}{}-{}{}-{}{}{}{}{}{}",
        hex[0],
        hex[1],
        hex[2],
        hex[3],
        hex[4],
        hex[5],
        hex[6],
        hex[7],
        hex[8],
        hex[9],
        hex[10],
        hex[11],
        hex[12],
        hex[13],
        hex[14],
        hex[15]
    )
}

// =============================================================================

#[cfg(test)]
mod tests {
    use super::{MarkerEvent, MarkerId, MarkerPhase};

    #[test]
    fn creates_uuid_shaped_marker_ids() {
        let id = MarkerId::new().to_string();

        assert_eq!(id.len(), 36);
        assert_eq!(&id[8..9], "-");
        assert_eq!(&id[13..14], "-");
        assert_eq!(&id[18..19], "-");
        assert_eq!(&id[23..24], "-");
    }

    #[test]
    fn stores_marker_metadata() {
        let marker = MarkerEvent::new(
            MarkerId::new(),
            "parse_document",
            MarkerPhase::Begin,
            Some("structural_parser".to_string()),
        );

        assert_eq!(marker.label, "parse_document");
        assert_eq!(marker.phase, MarkerPhase::Begin);
        assert_eq!(marker.origin.as_deref(), Some("structural_parser"));
    }
}
