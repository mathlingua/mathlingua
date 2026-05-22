use super::{CheckResult, check_in, init as init_collection, version as emit_version, view_in};
use crate::cli::Command;
use crate::environment::current_working_directory;
use crate::events::{EventLog, Level};
use std::io;
use std::path::{Path, PathBuf};

/// Stateful facade for running `mlg` commands with a shared event log.
pub struct Mlg {
    event_log: EventLog,
    working_directory: Option<PathBuf>,
}

impl Default for Mlg {
    fn default() -> Self {
        Self {
            event_log: EventLog::new(),
            working_directory: std::env::current_dir().ok(),
        }
    }
}

impl Mlg {
    /// Creates an `mlg` command facade with an empty event log.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an `mlg` command facade with an existing event log.
    pub fn with_event_log(event_log: EventLog) -> Self {
        Self {
            event_log,
            working_directory: std::env::current_dir().ok(),
        }
    }

    /// Creates an `mlg` command facade with an explicit working directory.
    pub fn with_working_directory(working_directory: impl Into<PathBuf>) -> Self {
        Self {
            event_log: EventLog::new(),
            working_directory: Some(working_directory.into()),
        }
    }

    /// Creates an `mlg` command facade with an existing event log and working directory.
    pub fn with_event_log_and_working_directory(
        event_log: EventLog,
        working_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            event_log,
            working_directory: Some(working_directory.into()),
        }
    }

    /// Returns the accumulated event log.
    pub fn event_log(&self) -> &EventLog {
        &self.event_log
    }

    /// Returns the accumulated event log mutably.
    pub fn event_log_mut(&mut self) -> &mut EventLog {
        &mut self.event_log
    }

    /// Consumes the facade and returns its event log.
    pub fn into_event_log(self) -> EventLog {
        self.event_log
    }

    /// Returns the working directory used by directory-sensitive commands.
    pub fn working_directory(&self) -> Option<&Path> {
        self.working_directory.as_deref()
    }

    /// Sets the working directory used by directory-sensitive commands.
    pub fn set_working_directory(&mut self, working_directory: impl Into<PathBuf>) -> &mut Self {
        self.working_directory = Some(working_directory.into());
        self
    }

    /// Runs a parsed CLI command and returns the process exit code it implies.
    pub fn run(&mut self, command: Command) -> i32 {
        match command {
            Command::Check(args) => self.run_check(&args.paths),
            Command::Init => self.run_init(),
            Command::Version => {
                self.version();
                0
            }
            Command::View(args) => self.run_view(args.port),
        }
    }

    /// Runs `mlg check` from this facade's working directory.
    pub fn check(&mut self, paths: &[PathBuf]) -> io::Result<CheckResult> {
        let cwd = self.resolve_working_directory()?;
        Ok(check_in(&cwd, paths, &mut self.event_log))
    }

    /// Runs `mlg check` from an explicit working directory.
    pub fn check_in(&mut self, cwd: &Path, paths: &[PathBuf]) -> CheckResult {
        check_in(cwd, paths, &mut self.event_log)
    }

    /// Runs `mlg init` in this facade's working directory.
    pub fn init(&mut self) -> io::Result<()> {
        let cwd = self.resolve_working_directory()?;
        init_collection(&cwd, &mut self.event_log)
    }

    /// Runs `mlg init` in an explicit root directory.
    pub fn init_in(&mut self, root: &Path) -> io::Result<()> {
        init_collection(root, &mut self.event_log)
    }

    /// Emits the compiled package version.
    pub fn version(&mut self) {
        emit_version(&mut self.event_log);
    }

    /// Starts `mlg view` from this facade's working directory.
    pub fn view(&mut self, port: u16) -> io::Result<()> {
        let cwd = self.resolve_working_directory()?;
        view_in(&cwd, port, &mut self.event_log)
    }

    /// Starts `mlg view` from an explicit working directory.
    pub fn view_in(&mut self, cwd: &Path, port: u16) -> io::Result<()> {
        view_in(cwd, port, &mut self.event_log)
    }

    /// Runs `mlg check` and returns the process exit code it implies.
    pub fn run_check(&mut self, paths: &[PathBuf]) -> i32 {
        let starting_event_count = self.event_log.events().len();
        let result = self.check(paths);
        self.exit_code_since(starting_event_count, result.is_err())
    }

    /// Runs `mlg init` and returns the process exit code it implies.
    pub fn run_init(&mut self) -> i32 {
        let starting_event_count = self.event_log.events().len();
        let result = self.init();
        self.exit_code_since(starting_event_count, result.is_err())
    }

    /// Runs `mlg view` and returns the process exit code it implies.
    pub fn run_view(&mut self, port: u16) -> i32 {
        let starting_event_count = self.event_log.events().len();
        let result = self.view(port);
        self.exit_code_since(starting_event_count, result.is_err())
    }

    fn resolve_working_directory(&mut self) -> io::Result<PathBuf> {
        if let Some(working_directory) = &self.working_directory {
            return Ok(working_directory.clone());
        }

        current_working_directory(&mut self.event_log)
            .ok_or_else(|| io::Error::other("Failed to determine the current working directory"))
    }

    fn exit_code_since(&self, starting_event_count: usize, command_failed: bool) -> i32 {
        i32::from(command_failed || self.has_errors_since(starting_event_count))
    }

    fn has_errors_since(&self, starting_event_count: usize) -> bool {
        self.event_log.events()[starting_event_count..]
            .iter()
            .filter_map(|event| event.as_message())
            .any(|message| message.level == Level::Error)
    }
}

// =============================================================================

#[cfg(test)]
mod tests {
    use super::Mlg;
    use crate::events::Event;
    use crate::mlg::config::default_config_contents;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn check_uses_the_configured_working_directory() {
        let temp_dir = TestDir::new();
        let root = temp_dir.path().join("repo");
        let content = root.join("content");
        let unrelated = temp_dir.path().join("unrelated");

        fs::create_dir_all(&content).unwrap();
        fs::create_dir(&unrelated).unwrap();
        fs::write(root.join("mlg.json"), default_config_contents()).unwrap();
        fs::write(content.join("sets.mlg"), "Title: \"Sets\"\n").unwrap();

        let mut mlg = Mlg::with_working_directory(&unrelated);
        mlg.set_working_directory(&content);

        let result = mlg.check(&[]).expect("check should resolve configured cwd");

        assert_eq!(mlg.working_directory(), Some(content.as_path()));
        assert_eq!(result.files_checked, 1);
        assert_eq!(
            user_events(mlg.event_log()),
            [Event::user_log("Checked 1 file").with_origin("mlg_check")]
        );
    }

    fn user_events(event_log: &crate::events::EventLog) -> Vec<Event> {
        event_log
            .events()
            .iter()
            .filter_map(|event| {
                event.as_message().and_then(|message| {
                    (message.audience == crate::events::Audience::User).then(|| event.clone())
                })
            })
            .collect()
    }

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "mlg-facade-test-{}-{}",
                std::process::id(),
                unique
            ));
            fs::create_dir(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
