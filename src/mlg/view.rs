use crate::diagnostics::DiagnosticTracker;

pub fn view(diagnostics: &mut DiagnosticTracker) {
    diagnostics.log("view");
}

#[cfg(test)]
mod tests {
    use super::view;
    use crate::diagnostics::{Diagnostic, DiagnosticTracker};

    #[test]
    fn returns_placeholder_view_output() {
        let mut diagnostics = DiagnosticTracker::new();

        view(&mut diagnostics);

        assert_eq!(diagnostics.diagnostics(), [Diagnostic::log("view")]);
    }
}
