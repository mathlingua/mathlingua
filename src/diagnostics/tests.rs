use super::{Diagnostic, DiagnosticTracker, Location, Severity};

fn sample_diagnostic() -> Diagnostic {
    Diagnostic {
        message: "unexpected token".to_string(),
        severity: Severity::Error,
        location: Location { row: 7 },
    }
}

#[test]
fn stores_a_clone_of_added_diagnostics() {
    let mut tracker = DiagnosticTracker::new();
    let diagnostic = sample_diagnostic();

    tracker.push(diagnostic.clone());

    let mut expected = sample_diagnostic();
    expected.message.push('!');

    assert_eq!(tracker.diagnostics(), [diagnostic]);
    assert_ne!(tracker.diagnostics(), [expected]);
}

#[test]
fn returns_a_clone_of_the_internal_collection() {
    let mut tracker = DiagnosticTracker::new();
    tracker.push(sample_diagnostic());

    let mut diagnostics = tracker.diagnostics().to_vec();
    diagnostics.clear();

    assert_eq!(tracker.diagnostics().len(), 1);
}

#[test]
fn supports_warning_diagnostics() {
    let diagnostic = Diagnostic::warning(3, "unused statement");

    let mut tracker = DiagnosticTracker::new();
    tracker.push(diagnostic.clone());

    assert_eq!(tracker.diagnostics(), [diagnostic]);
}

#[test]
fn reports_errors_only_for_error_diagnostics() {
    let mut tracker = DiagnosticTracker::new();
    tracker.push(Diagnostic::warning(3, "unused statement"));

    assert!(!tracker.has_errors());

    tracker.push(Diagnostic::error(4, "unexpected token"));

    assert!(tracker.has_errors());
}

#[test]
fn formats_display_using_one_based_line_numbers() {
    let diagnostic = Diagnostic::error(2, "unexpected token");

    assert_eq!(diagnostic.to_string(), "error at line 3: unexpected token");
}
