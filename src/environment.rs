use crate::diagnostics::DiagnosticTracker;
use std::env;
use std::path::PathBuf;

pub fn current_working_directory(tracker: &mut DiagnosticTracker) -> Option<PathBuf> {
    match env::current_dir() {
        Ok(cwd) => Some(cwd),
        Err(error) => {
            tracker.global_error(format!(
                "Failed to determine the current working directory: {error}"
            ));
            None
        }
    }
}

// =============================================================================

#[cfg(test)]
mod tests {
    use super::current_working_directory;
    use crate::diagnostics::DiagnosticTracker;

    #[test]
    fn returns_the_current_working_directory_without_emitting_diagnostics() {
        let mut tracker = DiagnosticTracker::new();

        let cwd = current_working_directory(&mut tracker);

        assert!(cwd.is_some());
        assert!(tracker.diagnostics().is_empty());
    }
}
