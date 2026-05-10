use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_MARKER_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MarkerId(String);

impl MarkerId {
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

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MarkerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkerPhase {
    Begin,
    End,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkerEvent {
    pub id: MarkerId,
    pub label: String,
    pub phase: MarkerPhase,
    pub origin: Option<String>,
}

impl MarkerEvent {
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

    pub fn with_origin_option(mut self, origin: Option<&str>) -> Self {
        self.origin = origin.map(str::to_owned);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkerRange {
    pub begin: MarkerEvent,
    pub end: MarkerEvent,
}

impl MarkerRange {
    pub fn new(begin: MarkerEvent, end: MarkerEvent) -> Self {
        Self { begin, end }
    }
}

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
