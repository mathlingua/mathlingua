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

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::MarkerId;
    use std::collections::HashSet;

    #[test]
    fn renders_a_uuid_shaped_string() {
        let id = MarkerId::new().to_string();

        assert_eq!(id.len(), 36);
        assert_eq!(&id[8..9], "-");
        assert_eq!(&id[13..14], "-");
        assert_eq!(&id[18..19], "-");
        assert_eq!(&id[23..24], "-");
    }

    #[test]
    fn sets_uuid_v4_version_and_variant_bits() {
        let id = MarkerId::new().to_string();

        // Version nibble at byte index 6 → string index 14.
        assert_eq!(&id[14..15], "4");
        // Variant nibble at byte index 8 → string index 19 ∈ {8, 9, a, b}.
        let variant = id.as_bytes()[19] as char;
        assert!(
            matches!(variant, '8' | '9' | 'a' | 'b'),
            "unexpected variant nibble: {variant}"
        );
    }

    #[test]
    fn as_str_and_display_agree() {
        let id = MarkerId::new();

        assert_eq!(id.as_str(), id.to_string());
    }

    #[test]
    fn default_produces_a_fresh_id() {
        let a = MarkerId::default();
        let b = MarkerId::default();

        assert_ne!(a, b);
    }

    #[test]
    fn successive_ids_are_unique() {
        let ids: HashSet<_> = (0..256).map(|_| MarkerId::new()).collect();

        assert_eq!(ids.len(), 256);
    }

    #[test]
    fn cloned_ids_compare_equal() {
        let id = MarkerId::new();

        assert_eq!(id.clone(), id);
    }
}
