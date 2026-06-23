use super::MarkerEvent;

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

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::MarkerRange;
    use crate::events::{MarkerEvent, MarkerId, MarkerPhase};

    fn begin_end_pair(label: &str) -> (MarkerEvent, MarkerEvent) {
        let id = MarkerId::new();
        let begin = MarkerEvent::new(id.clone(), label, MarkerPhase::Begin, None);
        let end = MarkerEvent::new(id, label, MarkerPhase::End, None);
        (begin, end)
    }

    #[test]
    fn stores_begin_and_end_events() {
        let (begin, end) = begin_end_pair("parse");
        let range = MarkerRange::new(begin.clone(), end.clone());

        assert_eq!(range.begin, begin);
        assert_eq!(range.end, end);
    }

    #[test]
    fn matching_pairs_share_marker_ids() {
        let (begin, end) = begin_end_pair("parse");
        let range = MarkerRange::new(begin, end);

        assert_eq!(range.begin.id, range.end.id);
        assert_eq!(range.begin.phase, MarkerPhase::Begin);
        assert_eq!(range.end.phase, MarkerPhase::End);
    }

    #[test]
    fn ranges_with_same_events_compare_equal() {
        let (begin, end) = begin_end_pair("scan");
        let a = MarkerRange::new(begin.clone(), end.clone());
        let b = MarkerRange::new(begin, end);

        assert_eq!(a, b);
    }

    #[test]
    fn ranges_with_different_ids_compare_unequal() {
        let (begin_a, end_a) = begin_end_pair("scan");
        let (begin_b, end_b) = begin_end_pair("scan");

        assert_ne!(
            MarkerRange::new(begin_a, end_a),
            MarkerRange::new(begin_b, end_b)
        );
    }
}
