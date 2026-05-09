use crate::diagnostics::DiagnosticTracker;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");

pub fn version(tracker: &mut DiagnosticTracker) {
    tracker.log(format!("{NAME}: {VERSION}"));
}

#[cfg(test)]
mod tests {
    use super::version;
    use crate::diagnostics::{Diagnostic, DiagnosticTracker};

    #[test]
    fn includes_package_name_and_version() {
        let mut tracker = DiagnosticTracker::new();

        version(&mut tracker);

        assert_eq!(
            tracker.diagnostics(),
            [Diagnostic::log(format!(
                "{}: {}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            ))]
        );
    }
}
