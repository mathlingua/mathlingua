use crate::backend::view::{CollectionView, build_collection_view};
use crate::environment::current_working_directory;
use crate::events::EventLog;
use crate::mlg::collection::resolve_collection_content_files;
use serde_json::to_writer_pretty;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Default port used by `mlg view` when the caller does not specify one.
const DEFAULT_PORT: u16 = 3000;
/// Event origin assigned to output emitted by the Next.js child process.
const NEXTJS_ORIGIN: &str = "nextjs";
/// Event origin used by the viewer command.
const ORIGIN: &str = "mlg_view";

/// Starts the collection viewer from the process current working directory.
pub fn view(event_log: &mut EventLog) -> io::Result<()> {
    let Some(cwd) = current_working_directory(event_log) else {
        return Err(io::Error::other(
            "Failed to determine the current working directory",
        ));
    };

    view_in(&cwd, DEFAULT_PORT, event_log)
}

/// Builds viewer data for the collection at `cwd` and starts the web viewer.
///
/// The backend writes a temporary JSON payload, launches the embedded Next.js
/// app with `MLG_VIEW_DATA_PATH` pointing at that payload, and keeps streaming
/// child output until the server exits.
pub fn view_in(cwd: &Path, port: u16, event_log: &mut EventLog) -> io::Result<()> {
    let files = resolve_collection_content_files(cwd, event_log, ORIGIN);
    if files.is_empty() {
        return finish_view_setup_with_possible_errors(event_log);
    }

    event_log.system_debug(
        Some(ORIGIN),
        format!("Building a rendered view for {} file(s)", files.len()),
    );

    let collection_view = match build_collection_view(cwd, &files, event_log) {
        Some(collection_view) => collection_view,
        None => {
            event_log.user_error(
                Some(ORIGIN),
                "View not started because one or more files could not be rendered",
            );
            return Err(io::Error::other(
                "One or more files could not be rendered for viewing",
            ));
        }
    };

    let view_session_dir = create_view_session_dir()?;
    let view_data_path = view_session_dir.join("collection.json");

    write_collection_view_data(&view_data_path, &collection_view)?;
    ensure_web_dependencies(event_log)?;

    let url = format!("http://localhost:{port}/");
    event_log.user_log(Some(ORIGIN), format!("Starting viewer at {url}"));

    let result = run_next_server(&view_data_path, port, &url, event_log);
    let _ = fs::remove_dir_all(&view_session_dir);
    result
}

/// Handles the case where no renderable files were found.
fn finish_view_setup_with_possible_errors(event_log: &mut EventLog) -> io::Result<()> {
    if event_log.has_errors() {
        Err(io::Error::other("Unable to start the viewer"))
    } else {
        event_log.user_log(Some(ORIGIN), "No Mathlingua files were found to render");
        Ok(())
    }
}

/// Installs web viewer dependencies when `web/node_modules` is absent.
fn ensure_web_dependencies(event_log: &mut EventLog) -> io::Result<()> {
    let web_dir = web_app_directory();
    if web_dir.join("node_modules").is_dir() {
        return Ok(());
    }

    event_log.user_log(Some(ORIGIN), "Installing web viewer dependencies");
    run_child(
        {
            let mut command = Command::new("npm");
            command.arg("install").current_dir(web_dir);
            command
        },
        NEXTJS_ORIGIN,
        event_log,
        None,
    )?;

    Ok(())
}

/// Starts the embedded Next.js development server for a generated view payload.
fn run_next_server(
    view_data_path: &Path,
    port: u16,
    url: &str,
    event_log: &mut EventLog,
) -> io::Result<()> {
    let mut command = Command::new("npm");
    command
        .arg("run")
        .arg("dev")
        .arg("--")
        .arg("--hostname")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(port.to_string())
        .current_dir(web_app_directory())
        .env("MLG_VIEW_DATA_PATH", view_data_path)
        .env("NEXT_TELEMETRY_DISABLED", "1");

    run_child(command, NEXTJS_ORIGIN, event_log, Some(url))
}

/// Runs a child process while streaming stdout/stderr into the event log.
///
/// When `ready_url` is supplied, the child must print a known Next.js ready line
/// before exiting or the run is treated as a startup failure.
fn run_child(
    mut command: Command,
    origin: &str,
    event_log: &mut EventLog,
    ready_url: Option<&str>,
) -> io::Result<()> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            event_log.user_error(
                Some(ORIGIN),
                format!("Failed to start the web viewer process: {error}"),
            );
            return Err(error);
        }
    };

    let (sender, receiver) = mpsc::channel();
    let stdout_thread = child
        .stdout
        .take()
        .map(|stdout| spawn_output_reader(stdout, OutputStream::Stdout, sender.clone()));
    let stderr_thread = child
        .stderr
        .take()
        .map(|stderr| spawn_output_reader(stderr, OutputStream::Stderr, sender));

    let outcome = stream_child_output(&mut child, receiver, origin, event_log, ready_url);
    join_output_thread(stdout_thread, event_log);
    join_output_thread(stderr_thread, event_log);
    let outcome = outcome?;

    if !outcome.was_ready && ready_url.is_some() {
        event_log.user_error(Some(ORIGIN), "The web viewer exited before it became ready");
        return Err(io::Error::other(
            "The web viewer exited before it became ready",
        ));
    }

    if outcome.status.success() {
        Ok(())
    } else {
        event_log.user_error(
            Some(ORIGIN),
            format!("The web viewer exited with status {}", outcome.status),
        );
        Err(io::Error::other(format!(
            "The web viewer exited with status {}",
            outcome.status
        )))
    }
}

/// Streams child-process output until the child exits.
fn stream_child_output(
    child: &mut Child,
    receiver: Receiver<OutputLine>,
    origin: &str,
    event_log: &mut EventLog,
    ready_url: Option<&str>,
) -> io::Result<ChildOutcome> {
    let mut was_ready = false;

    loop {
        match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(line) => {
                let is_ready = is_ready_line(&line.text);
                if is_ready
                    && !was_ready
                    && let Some(url) = ready_url
                {
                    event_log.user_log(Some(ORIGIN), format!("Viewer ready at {url}"));
                }
                was_ready |= is_ready;
                log_child_output(line, origin, event_log);
            }
            Err(RecvTimeoutError::Timeout) => {
                if let Some(status) = child.try_wait()? {
                    while let Ok(line) = receiver.try_recv() {
                        let is_ready = is_ready_line(&line.text);
                        if is_ready
                            && !was_ready
                            && let Some(url) = ready_url
                        {
                            event_log.user_log(Some(ORIGIN), format!("Viewer ready at {url}"));
                        }
                        was_ready |= is_ready;
                        log_child_output(line, origin, event_log);
                    }
                    return Ok(ChildOutcome { status, was_ready });
                }
            }
            Err(RecvTimeoutError::Disconnected) => {
                return child
                    .wait()
                    .map(|status| ChildOutcome { status, was_ready });
            }
        }
    }
}

/// Re-emits one child-process output line as a system event.
fn log_child_output(line: OutputLine, origin: &str, event_log: &mut EventLog) {
    match line.stream {
        OutputStream::Stdout => event_log.system_log(Some(origin), line.text),
        OutputStream::Stderr => event_log.system_log(Some(origin), line.text),
    }
}

/// Spawns a thread that reads one child-process stream line by line.
fn spawn_output_reader(
    stream: impl io::Read + Send + 'static,
    output_stream: OutputStream,
    sender: Sender<OutputLine>,
) -> JoinHandle<io::Result<()>> {
    thread::spawn(move || {
        let reader = BufReader::new(stream);
        for line in reader.lines() {
            let line = line?;
            if sender
                .send(OutputLine {
                    stream: output_stream,
                    text: line,
                })
                .is_err()
            {
                break;
            }
        }

        Ok(())
    })
}

/// Joins an output-reader thread and reports any reader failures.
fn join_output_thread(thread: Option<JoinHandle<io::Result<()>>>, event_log: &mut EventLog) {
    let Some(thread) = thread else {
        return;
    };

    match thread.join() {
        Ok(Ok(())) => {}
        Ok(Err(error)) => {
            event_log.system_warning(
                Some(ORIGIN),
                format!("Failed to read web viewer output: {error}"),
            );
        }
        Err(_) => {
            event_log.system_warning(Some(ORIGIN), "A web viewer output thread panicked");
        }
    }
}

/// Writes the collection view JSON consumed by the web app.
fn write_collection_view_data(path: &Path, collection_view: &CollectionView) -> io::Result<()> {
    let file = fs::File::create(path)?;
    to_writer_pretty(file, collection_view)
        .map_err(|error| io::Error::other(format!("Failed to write view data: {error}")))
}

/// Creates a unique temporary directory for one viewer session.
fn create_view_session_dir() -> io::Result<PathBuf> {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("mlg-view-{}-{}", std::process::id(), unique));
    fs::create_dir(&path)?;
    Ok(path)
}

/// Returns the embedded web app directory inside this crate.
fn web_app_directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("web")
}

/// Detects Next.js startup readiness lines.
fn is_ready_line(text: &str) -> bool {
    text.contains("Ready in") || text.contains("Local:")
}

/// Identifies which child stream produced an output line.
#[derive(Clone, Copy)]
enum OutputStream {
    /// Child stdout.
    Stdout,
    /// Child stderr.
    Stderr,
}

/// One line of output read from the child process.
struct OutputLine {
    /// Stream that produced the line.
    stream: OutputStream,
    /// Text of the line without its trailing newline.
    text: String,
}

/// Final state of a child process after its output has been streamed.
struct ChildOutcome {
    /// Exit status returned by the child.
    status: ExitStatus,
    /// Whether a readiness line was seen before exit.
    was_ready: bool,
}

// =============================================================================

#[cfg(test)]
mod tests {
    use super::{create_view_session_dir, web_app_directory, write_collection_view_data};
    use crate::backend::view::{CollectionView, FileView, GroupView, SectionView};
    use std::fs;

    #[test]
    fn writes_collection_view_data_as_json() {
        let dir = create_view_session_dir().expect("expected temp dir");
        let path = dir.join("collection.json");
        let collection = CollectionView {
            title: "demo".to_string(),
            files: vec![FileView {
                path: "content/example.mlg".to_string(),
                items: vec![GroupView {
                    kind: "Title".to_string(),
                    heading: None,
                    heading_latex: None,
                    sections: vec![SectionView {
                        label: "Title".to_string(),
                        inline_argument: Some("\"Example\"".to_string()),
                        inline_latex: None,
                        arguments: vec![],
                    }],
                }],
            }],
        };

        write_collection_view_data(&path, &collection).expect("expected json file");

        let contents = fs::read_to_string(&path).expect("expected collection data");
        assert!(contents.contains("\"title\": \"demo\""));
        assert!(contents.contains("\"path\": \"content/example.mlg\""));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn points_to_the_embedded_web_app_directory() {
        assert!(web_app_directory().join("package.json").is_file());
    }
}
