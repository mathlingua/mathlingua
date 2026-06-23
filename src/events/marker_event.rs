use super::MarkerId;

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

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::{MarkerEvent, MarkerId, MarkerPhase};

    #[test]
    fn stores_marker_metadata() {
        let id = MarkerId::new();
        let marker = MarkerEvent::new(
            id.clone(),
            "parse_document",
            MarkerPhase::Begin,
            Some("structural_parser".to_string()),
        );

        assert_eq!(marker.id, id);
        assert_eq!(marker.label, "parse_document");
        assert_eq!(marker.phase, MarkerPhase::Begin);
        assert_eq!(marker.origin.as_deref(), Some("structural_parser"));
    }

    #[test]
    fn accepts_label_as_string_slice_or_owned() {
        let from_slice = MarkerEvent::new(MarkerId::new(), "scan", MarkerPhase::End, None);
        let from_owned =
            MarkerEvent::new(MarkerId::new(), "scan".to_string(), MarkerPhase::End, None);

        assert_eq!(from_slice.label, from_owned.label);
    }

    #[test]
    fn with_origin_option_replaces_existing_origin() {
        let marker = MarkerEvent::new(
            MarkerId::new(),
            "typecheck",
            MarkerPhase::Begin,
            Some("old".to_string()),
        )
        .with_origin_option(Some("new"));

        assert_eq!(marker.origin.as_deref(), Some("new"));
    }

    #[test]
    fn with_origin_option_clears_origin_when_none() {
        let marker = MarkerEvent::new(
            MarkerId::new(),
            "typecheck",
            MarkerPhase::End,
            Some("old".to_string()),
        )
        .with_origin_option(None);

        assert!(marker.origin.is_none());
    }

    #[test]
    fn phases_are_distinct() {
        assert_ne!(MarkerPhase::Begin, MarkerPhase::End);
    }

    #[test]
    fn cloned_marker_events_compare_equal() {
        let marker = MarkerEvent::new(MarkerId::new(), "load", MarkerPhase::Begin, None);

        assert_eq!(marker.clone(), marker);
    }
}
