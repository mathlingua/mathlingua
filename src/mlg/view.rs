use crate::diagnostics::DiagnosticTracker;

pub fn view(tracker: &mut DiagnosticTracker) {
    tracker.log("view");
}

#[cfg(test)]
mod tests {
    use super::view;
    use crate::diagnostics::{Diagnostic, DiagnosticTracker};

    #[test]
    fn returns_placeholder_view_output() {
        let mut tracker = DiagnosticTracker::new();

        view(&mut tracker);

        assert_eq!(tracker.diagnostics(), [Diagnostic::log("view")]);
    }
}
