use crate::backend::collection::SourceCollection;
use crate::backend::view::CollectionView;
use crate::events::{Event, EventLog, EventLogListener};
use crate::mlg::util::{has_blocking_user_issues_since, no_errors_since};
use serde_json::to_writer_pretty;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc::{self, Receiver, RecvTimeoutError, Sender},
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const NEXTJS_ORIGIN: &str = "nextjs";
const ORIGIN: &str = "mlg_view";
const VIEW_BIND_HOST: &str = "0.0.0.0";
static NEXT_VIEW_SESSION_DIR_ID: AtomicUsize = AtomicUsize::new(0);

pub struct ViewResult {
    pub event_log: EventLog,
    pub successful: bool,
}

pub fn view(cwd: &Path, port: u16, listener: Option<Box<dyn EventLogListener>>) -> ViewResult {
    let mut event_log = EventLog::new();
    if let Some(listener) = listener {
        event_log.add_boxed_listener(listener);
    }

    let starting_event_count = event_log.events().len();
    let io_ok = view_in(cwd, port, &mut event_log).is_ok();
    let successful = io_ok && no_errors_since(&event_log, starting_event_count);

    ViewResult {
        event_log,
        successful,
    }
}

pub(super) fn view_in(cwd: &Path, port: u16, event_log: &mut EventLog) -> io::Result<()> {
    let starting_event_count = event_log.events().len();
    let mut collection = SourceCollection::load(cwd, event_log, ORIGIN);
    if collection.source_files().is_empty() {
        return finish_view_setup_with_possible_errors(event_log);
    }

    event_log.system_debug(
        Some(ORIGIN),
        format!(
            "Checking collection before rendering {} file(s)",
            collection.source_files().len()
        ),
    );

    collection.run_check_passes(event_log, ORIGIN);

    if has_blocking_user_issues_since(event_log, starting_event_count) {
        event_log.user_error(
            Some(ORIGIN),
            "View not started because one or more files could not be rendered",
        );
        return Err(io::Error::other(
            "One or more files could not be rendered for viewing",
        ));
    }

    event_log.system_debug(
        Some(ORIGIN),
        format!(
            "Building a rendered view for {} file(s)",
            collection.parsed_files().len()
        ),
    );

    let Some(collection_view) = collection.build_view(event_log) else {
        event_log.user_error(
            Some(ORIGIN),
            "View not started because one or more files could not be rendered",
        );
        return Err(io::Error::other(
            "One or more files could not be rendered for viewing",
        ));
    };

    let view_session_dir = create_view_session_dir()?;
    let view_data_path = view_session_dir.join("collection.json");

    write_collection_view_data(&view_data_path, &collection_view)?;
    ensure_web_dependencies(event_log)?;

    check_port_is_available(port, event_log)?;

    let url = format!("http://localhost:{port}/");
    event_log.user_log(Some(ORIGIN), format!("Starting viewer at {url}"));

    let (refresh_sender, refresh_receiver) = mpsc::channel();
    let stop_refresh = Arc::new(AtomicBool::new(false));
    let refresh_thread = spawn_view_data_refresher(
        cwd.to_path_buf(),
        view_data_path.clone(),
        Arc::clone(&stop_refresh),
        refresh_sender,
    );
    let result = run_next_server(&view_data_path, port, &url, refresh_receiver, event_log);
    stop_refresh.store(true, Ordering::Relaxed);
    join_view_data_refresher(refresh_thread, event_log);
    let _ = fs::remove_dir_all(&view_session_dir);
    result
}

/// Reports an unusable port before the web viewer is started.
///
/// Binding here is a pre-flight probe: the listener is dropped immediately and
/// the real server binds the port a moment later. Something else can still take
/// the port in that gap, so this is a way to give a clear message in the common
/// case, not a guarantee — [`describe_child_startup_failure`] covers the rest.
fn check_port_is_available(port: u16, event_log: &mut EventLog) -> io::Result<()> {
    let error = match TcpListener::bind((VIEW_BIND_HOST, port)) {
        Ok(_) => return Ok(()),
        Err(error) => error,
    };

    let message = match error.kind() {
        io::ErrorKind::AddrInUse => format!(
            "Port {port} is already in use by another program. \
             Stop that program, or start the viewer on a different port with `mlg view --port <PORT>`"
        ),
        io::ErrorKind::PermissionDenied => format!(
            "Port {port} cannot be opened because this user is not permitted to use it. \
             Ports below 1024 usually require elevated privileges, so try `mlg view --port <PORT>` with a port above 1024"
        ),
        _ => format!("Port {port} is not available for the viewer: {error}"),
    };

    event_log.user_error(Some(ORIGIN), message.clone());
    Err(io::Error::new(error.kind(), message))
}

fn finish_view_setup_with_possible_errors(event_log: &mut EventLog) -> io::Result<()> {
    if event_log.has_errors() {
        Err(io::Error::other("Unable to start the viewer"))
    } else {
        event_log.user_log(Some(ORIGIN), "No Mathlingua files were found to render");
        Ok(())
    }
}

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
        None,
        None,
    )?;

    Ok(())
}

fn run_next_server(
    view_data_path: &Path,
    port: u16,
    url: &str,
    refresh_receiver: Receiver<Vec<Event>>,
    event_log: &mut EventLog,
) -> io::Result<()> {
    let mut command = Command::new("npm");
    command
        .arg("run")
        .arg("dev")
        .arg("--")
        .arg("--hostname")
        .arg(VIEW_BIND_HOST)
        .arg("--port")
        .arg(port.to_string())
        .current_dir(web_app_directory())
        .env("MLG_VIEW_DATA_PATH", view_data_path)
        .env("NEXT_TELEMETRY_DISABLED", "1");

    run_child(
        command,
        NEXTJS_ORIGIN,
        event_log,
        Some(url),
        Some(refresh_receiver),
        Some(port),
    )
}

fn run_child(
    mut command: Command,
    origin: &str,
    event_log: &mut EventLog,
    ready_url: Option<&str>,
    refresh_receiver: Option<Receiver<Vec<Event>>>,
    port: Option<u16>,
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
        .map(|stdout| spawn_output_reader(stdout, sender.clone()));
    let stderr_thread = child
        .stderr
        .take()
        .map(|stderr| spawn_output_reader(stderr, sender));

    let outcome = stream_child_output(
        &mut child,
        receiver,
        refresh_receiver,
        origin,
        event_log,
        ready_url,
    );
    join_output_thread(stdout_thread, event_log);
    join_output_thread(stderr_thread, event_log);
    let outcome = outcome?;

    if !outcome.was_ready && ready_url.is_some() {
        let message = describe_child_startup_failure(&outcome, port);
        event_log.user_error(Some(ORIGIN), message.clone());
        return Err(io::Error::other(message));
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

fn stream_child_output(
    child: &mut Child,
    output_receiver: Receiver<OutputLine>,
    refresh_receiver: Option<Receiver<Vec<Event>>>,
    origin: &str,
    event_log: &mut EventLog,
    ready_url: Option<&str>,
) -> io::Result<ChildOutcome> {
    let mut was_ready = false;
    let mut recent_output: Vec<String> = Vec::new();

    let handle_line = |line: OutputLine,
                           was_ready: &mut bool,
                           recent_output: &mut Vec<String>,
                           event_log: &mut EventLog| {
        let is_ready = is_ready_line(&line.text);
        if is_ready
            && !*was_ready
            && let Some(url) = ready_url
        {
            event_log.user_log(Some(ORIGIN), format!("Viewer ready at {url}"));
        }
        *was_ready |= is_ready;

        if recent_output.len() == RECENT_OUTPUT_LIMIT {
            recent_output.remove(0);
        }
        recent_output.push(line.text.clone());

        log_child_output(line, origin, event_log);
    };

    loop {
        drain_refresh_events(refresh_receiver.as_ref(), event_log);

        match output_receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(line) => handle_line(line, &mut was_ready, &mut recent_output, event_log),
            Err(RecvTimeoutError::Timeout) => {
                drain_refresh_events(refresh_receiver.as_ref(), event_log);
                if let Some(status) = child.try_wait()? {
                    while let Ok(line) = output_receiver.try_recv() {
                        handle_line(line, &mut was_ready, &mut recent_output, event_log);
                    }
                    drain_refresh_events(refresh_receiver.as_ref(), event_log);
                    return Ok(ChildOutcome {
                        status,
                        was_ready,
                        recent_output,
                    });
                }
            }
            Err(RecvTimeoutError::Disconnected) => {
                drain_refresh_events(refresh_receiver.as_ref(), event_log);
                return child.wait().map(|status| ChildOutcome {
                    status,
                    was_ready,
                    recent_output,
                });
            }
        }
    }
}

fn drain_refresh_events(receiver: Option<&Receiver<Vec<Event>>>, event_log: &mut EventLog) {
    let Some(receiver) = receiver else {
        return;
    };

    while let Ok(events) = receiver.try_recv() {
        for event in events {
            event_log.push(event);
        }
    }
}

fn log_child_output(line: OutputLine, origin: &str, event_log: &mut EventLog) {
    event_log.system_log(Some(origin), line.text);
}

fn spawn_output_reader(
    stream: impl io::Read + Send + 'static,
    sender: Sender<OutputLine>,
) -> JoinHandle<io::Result<()>> {
    thread::spawn(move || {
        let reader = BufReader::new(stream);
        for line in reader.lines() {
            let line = line?;
            if sender.send(OutputLine { text: line }).is_err() {
                break;
            }
        }

        Ok(())
    })
}

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

fn write_collection_view_data(path: &Path, collection_view: &CollectionView) -> io::Result<()> {
    let temp_path = temporary_view_data_path(path);
    let result = (|| {
        let file = fs::File::create(&temp_path)?;
        to_writer_pretty(file, collection_view)
            .map_err(|error| io::Error::other(format!("Failed to write view data: {error}")))?;
        fs::rename(&temp_path, path)?;
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }

    result
}

fn temporary_view_data_path(path: &Path) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    path.with_extension(format!("json.tmp-{}-{unique}", std::process::id()))
}

fn spawn_view_data_refresher(
    cwd: PathBuf,
    view_data_path: PathBuf,
    stop: Arc<AtomicBool>,
    diagnostics: Sender<Vec<Event>>,
) -> JoinHandle<io::Result<()>> {
    thread::spawn(move || {
        let mut last_fingerprint = view_source_fingerprint(&cwd);

        while !stop.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(250));
            let fingerprint = view_source_fingerprint(&cwd);
            if fingerprint == last_fingerprint {
                continue;
            }

            last_fingerprint = fingerprint;
            match rebuild_collection_view_data(&cwd, &view_data_path) {
                Ok(ViewDataRefresh::Updated) => {}
                Ok(ViewDataRefresh::Blocked(events)) => {
                    if diagnostics.send(events).is_err() {
                        break;
                    }
                }
                Err(error) => {
                    let event = Event::user_error(format!(
                        "Rendered view was not updated because the view data could not be written: {error}"
                    ))
                    .with_origin(ORIGIN);
                    if diagnostics.send(vec![event]).is_err() {
                        break;
                    }
                }
            }
        }

        Ok(())
    })
}

fn join_view_data_refresher(thread: JoinHandle<io::Result<()>>, event_log: &mut EventLog) {
    match thread.join() {
        Ok(Ok(())) => {}
        Ok(Err(error)) => {
            event_log.system_warning(
                Some(ORIGIN),
                format!("Failed to refresh rendered view data: {error}"),
            );
        }
        Err(_) => {
            event_log.system_warning(Some(ORIGIN), "The rendered view refresher thread panicked");
        }
    }
}

fn rebuild_collection_view_data(cwd: &Path, view_data_path: &Path) -> io::Result<ViewDataRefresh> {
    let mut event_log = EventLog::new();
    let starting_event_count = event_log.events().len();
    let mut collection = SourceCollection::load(cwd, &mut event_log, ORIGIN);
    collection.run_check_passes(&mut event_log, ORIGIN);

    if has_blocking_user_issues_since(&event_log, starting_event_count) {
        event_log.user_error(
            Some(ORIGIN),
            "Rendered view was not updated because the current MathLingua sources have errors",
        );
        return Ok(ViewDataRefresh::Blocked(event_log.events().to_vec()));
    }

    let Some(collection_view) = collection.build_view(&mut event_log) else {
        event_log.user_error(
            Some(ORIGIN),
            "Rendered view was not updated because one or more files could not be rendered",
        );
        return Ok(ViewDataRefresh::Blocked(event_log.events().to_vec()));
    };

    write_collection_view_data(view_data_path, &collection_view)?;
    Ok(ViewDataRefresh::Updated)
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ViewDataRefresh {
    Updated,
    Blocked(Vec<Event>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ViewSourceFingerprint {
    root: PathBuf,
    files: Vec<ViewFileFingerprint>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ViewFileFingerprint {
    path: PathBuf,
    len: u64,
    modified: SystemTime,
}

fn view_source_fingerprint(cwd: &Path) -> ViewSourceFingerprint {
    let mut event_log = EventLog::new();
    let collection = SourceCollection::load(cwd, &mut event_log, ORIGIN);
    let files = collection
        .source_files()
        .iter()
        .chain(collection.toc_files().iter())
        .map(|path| view_file_fingerprint(path))
        .collect();

    ViewSourceFingerprint {
        root: collection.root().to_path_buf(),
        files,
    }
}

fn view_file_fingerprint(path: &Path) -> ViewFileFingerprint {
    let metadata = fs::metadata(path).ok();
    ViewFileFingerprint {
        path: path.to_path_buf(),
        len: metadata.as_ref().map_or(0, fs::Metadata::len),
        modified: metadata
            .and_then(|metadata| metadata.modified().ok())
            .unwrap_or(UNIX_EPOCH),
    }
}

fn create_view_session_dir() -> io::Result<PathBuf> {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let id = NEXT_VIEW_SESSION_DIR_ID.fetch_add(1, Ordering::Relaxed);
    let path =
        std::env::temp_dir().join(format!("mlg-view-{}-{}-{}", std::process::id(), unique, id));
    fs::create_dir(&path)?;
    Ok(path)
}

fn web_app_directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("web")
}

fn is_ready_line(text: &str) -> bool {
    text.contains("Ready in") || text.contains("Local:")
}

struct OutputLine {
    text: String,
}

struct ChildOutcome {
    status: ExitStatus,
    was_ready: bool,
    /// The tail of the child's combined stdout/stderr, used to explain a startup
    /// failure. Bounded so a chatty child cannot grow this without limit.
    recent_output: Vec<String>,
}

/// How many trailing output lines are kept to diagnose a startup failure.
const RECENT_OUTPUT_LIMIT: usize = 50;

/// How many output lines are quoted back to the user for an unrecognized failure.
const REPORTED_OUTPUT_LINES: usize = 5;

/// Explains why the web viewer exited before it was ready, in the user's terms.
///
/// The child is a Next.js process, so the specifics arrive only as text on its
/// output. Recognized causes get an actionable message; anything else falls back
/// to quoting the child's own last words, which beats a bare "it exited".
fn describe_child_startup_failure(outcome: &ChildOutcome, port: Option<u16>) -> String {
    let output = outcome.recent_output.join("\n");

    if output.contains("EADDRINUSE") || output.contains("address already in use") {
        return match port {
            Some(port) => format!(
                "The viewer could not start because port {port} is already in use by another program. \
                 Stop that program, or start the viewer on a different port with `mlg view --port <PORT>`"
            ),
            None => "The viewer could not start because its port is already in use by another program"
                .to_string(),
        };
    }

    if output.contains("EACCES") || output.contains("permission denied") {
        return match port {
            Some(port) => format!(
                "The viewer could not start because it is not permitted to open port {port}. \
                 Ports below 1024 usually require elevated privileges, so try `mlg view --port <PORT>` with a port above 1024"
            ),
            None => "The viewer could not start because it was denied permission".to_string(),
        };
    }

    let details = outcome
        .recent_output
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .rev()
        .take(REPORTED_OUTPUT_LINES)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n  ");

    if details.is_empty() {
        return format!(
            "The web viewer exited before it became ready, without reporting a reason \
             (it exited with status {})",
            outcome.status
        );
    }

    format!("The web viewer exited before it became ready. It reported:\n  {details}")
}

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::{
        ChildOutcome, VIEW_BIND_HOST, ViewDataRefresh, check_port_is_available,
        create_view_session_dir, describe_child_startup_failure, rebuild_collection_view_data,
        web_app_directory, write_collection_view_data,
    };
    use crate::backend::view::{CollectionView, FileView, GroupView, PageView, SectionView};
    use crate::events::EventLog;
    use serde_json::Value;
    use std::fs;
    use std::net::TcpListener;

    fn outcome_with_output(lines: &[&str]) -> ChildOutcome {
        ChildOutcome {
            status: std::process::Command::new("sh")
                .arg("-c")
                .arg("exit 1")
                .status()
                .expect("expected a child status"),
            was_ready: false,
            recent_output: lines.iter().map(|line| line.to_string()).collect(),
        }
    }

    #[test]
    fn reports_an_occupied_port_as_being_in_use() {
        let listener = TcpListener::bind((VIEW_BIND_HOST, 0)).expect("expected a bound port");
        let port = listener.local_addr().expect("expected an address").port();
        let mut event_log = EventLog::new();

        let error = check_port_is_available(port, &mut event_log)
            .expect_err("expected the occupied port to be rejected");

        assert!(error.to_string().contains(&format!("Port {port}")));
        assert!(error.to_string().contains("already in use"));
        assert!(error.to_string().contains("mlg view --port"));
        assert!(
            event_log
                .events()
                .iter()
                .filter_map(|event| event.as_message())
                .any(|message| message.message.contains("already in use"))
        );
    }

    #[test]
    fn accepts_a_free_port() {
        // An ephemeral port can be claimed by a concurrently running test between
        // being released here and being probed, so a few attempts are allowed
        // before the check itself is blamed.
        for attempt in 0..8 {
            let port = {
                let listener =
                    TcpListener::bind((VIEW_BIND_HOST, 0)).expect("expected a bound port");
                listener.local_addr().expect("expected an address").port()
            };
            let mut event_log = EventLog::new();

            if check_port_is_available(port, &mut event_log).is_ok() {
                assert!(event_log.events().is_empty());
                return;
            }

            assert!(attempt < 7, "a freed port was never reported as available");
        }
    }

    #[test]
    fn explains_an_address_in_use_startup_failure() {
        let outcome = outcome_with_output(&[
            " ⨯ Failed to start server",
            "Error: listen EADDRINUSE: address already in use 0.0.0.0:3999",
        ]);

        let message = describe_child_startup_failure(&outcome, Some(3999));

        assert!(message.contains("port 3999 is already in use"));
        assert!(message.contains("mlg view --port"));
    }

    #[test]
    fn explains_a_permission_denied_startup_failure() {
        let outcome =
            outcome_with_output(&["Error: listen EACCES: permission denied 0.0.0.0:80"]);

        let message = describe_child_startup_failure(&outcome, Some(80));

        assert!(message.contains("not permitted to open port 80"));
        assert!(message.contains("above 1024"));
    }

    #[test]
    fn quotes_the_child_output_for_an_unrecognized_startup_failure() {
        let outcome = outcome_with_output(&[
            "",
            "> dev",
            "Error: Cannot find module 'next'",
            "   at Module._resolveFilename",
        ]);

        let message = describe_child_startup_failure(&outcome, Some(3000));

        assert!(message.contains("exited before it became ready"));
        assert!(message.contains("Cannot find module 'next'"));
        // Blank lines are dropped rather than padding the quoted output.
        assert!(!message.contains("\n  \n"));
    }

    #[test]
    fn reports_a_silent_startup_failure_without_quoting_empty_output() {
        let outcome = outcome_with_output(&[]);

        let message = describe_child_startup_failure(&outcome, Some(3000));

        assert!(message.contains("without reporting a reason"));
    }

    #[test]
    fn writes_collection_view_data_as_json() {
        let dir = create_view_session_dir().expect("expected temp dir");
        let path = dir.join("collection.json");
        let collection = CollectionView {
            title: "demo".to_string(),
            directories: vec![],
            files: vec![FileView {
                path: "content/example.mlg".to_string(),
                title: None,
                items: vec![GroupView {
                    id: "18582990-701a-40d3-8ce3-ae12bd08a561".to_string(),
                    kind: "Title".to_string(),
                    definition_keys: vec![],
                    heading: None,
                    heading_latex: None,
                    body_text: None,
                    page: Some(PageView {
                        kind: "Title".to_string(),
                        text: "Example".to_string(),
                    }),
                    source: "Title: \"Example\"".to_string(),
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

    #[test]
    fn viewer_binds_to_all_network_interfaces() {
        assert_eq!(VIEW_BIND_HOST, "0.0.0.0");
    }

    #[test]
    fn rebuilds_collection_view_data_from_current_source_files() {
        let dir = create_view_session_dir().expect("expected temp dir");
        let root = dir.join("collection");
        let content = root.join("content");
        let file = content.join("sets.mlg");
        let path = dir.join("collection.json");

        fs::create_dir_all(&content).expect("expected content dir");
        fs::write(
            &file,
            "[\\set]\nDescribes: S\nDocumented:\n. called: \"set\"\n",
        )
        .expect("expected source file");

        assert_eq!(
            rebuild_collection_view_data(&root, &path).expect("expected initial view data"),
            ViewDataRefresh::Updated
        );
        let contents = fs::read_to_string(&path).expect("expected initial data");
        assert!(contents.contains("\\\\textrm{set}"));

        fs::write(
            &file,
            "[\\set]\nDescribes: S\nDocumented:\n. called: \"updated set\"\n",
        )
        .expect("expected updated source file");

        assert_eq!(
            rebuild_collection_view_data(&root, &path).expect("expected refreshed view data"),
            ViewDataRefresh::Updated
        );
        let contents = fs::read_to_string(&path).expect("expected refreshed data");
        assert!(contents.contains("\\\\textrm{updated set}"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rebuild_collection_view_data_applies_toc_titles_order_and_hidden_files() {
        let dir = create_view_session_dir().expect("expected temp dir");
        let root = dir.join("collection");
        let content = root.join("content");
        let visible_dir = content.join("visible_dir");
        let hidden_dir = content.join("hidden_dir");
        let path = dir.join("collection.json");

        fs::create_dir_all(&visible_dir).expect("expected visible dir");
        fs::create_dir_all(&hidden_dir).expect("expected hidden dir");
        fs::write(content.join("alpha_file.mlg"), "Title: \"Alpha\"\n")
            .expect("expected alpha source file");
        fs::write(content.join("gamma_file.mlg"), "Title: \"Gamma\"\n")
            .expect("expected gamma source file");
        fs::write(content.join("hidden_file.mlg"), "Title: \"Hidden\"\n")
            .expect("expected hidden source file");
        fs::write(visible_dir.join("inside.mlg"), "Title: \"Inside\"\n")
            .expect("expected visible directory source file");
        fs::write(hidden_dir.join("inside.mlg"), "Title: \"Hidden Inside\"\n")
            .expect("expected hidden directory source file");
        fs::write(
            content.join("toc"),
            "gamma_file.mlg -> Custom Gamma\nvisible_dir -> Visible Directory\nhidden_dir -> HIDDEN\nhidden_file.mlg -> HIDDEN\nalpha_file.mlg\n",
        )
        .expect("expected toc file");

        assert_eq!(
            rebuild_collection_view_data(&root, &path).expect("expected view data"),
            ViewDataRefresh::Updated
        );

        let contents = fs::read_to_string(&path).expect("expected collection data");
        let json: Value = serde_json::from_str(&contents).expect("expected json");
        let directories = json["directories"]
            .as_array()
            .expect("expected directories");
        let files = json["files"].as_array().expect("expected files");

        assert_eq!(directories.len(), 1);
        assert_eq!(directories[0]["path"], "content/visible_dir");
        assert_eq!(directories[0]["title"], "Visible Directory");
        assert_eq!(files.len(), 3);
        assert_eq!(files[0]["path"], "content/gamma_file.mlg");
        assert_eq!(files[0]["title"], "Custom Gamma");
        assert_eq!(files[1]["path"], "content/visible_dir/inside.mlg");
        assert_eq!(files[2]["path"], "content/alpha_file.mlg");
        assert!(files[2]["title"].is_null());
        assert!(!contents.contains("hidden_file.mlg"));
        assert!(!contents.contains("hidden_dir"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn reports_refresh_errors_without_replacing_last_good_view_data() {
        let dir = create_view_session_dir().expect("expected temp dir");
        let root = dir.join("collection");
        let content = root.join("content");
        let file = content.join("sets.mlg");
        let path = dir.join("collection.json");

        fs::create_dir_all(&content).expect("expected content dir");
        fs::write(
            &file,
            "[\\set]\nDescribes: S\nDocumented:\n. called: \"set\"\n",
        )
        .expect("expected source file");

        assert_eq!(
            rebuild_collection_view_data(&root, &path).expect("expected initial view data"),
            ViewDataRefresh::Updated
        );

        fs::write(&file, "[\\set]\nDescribes: S\n").expect("expected invalid source file");

        let outcome =
            rebuild_collection_view_data(&root, &path).expect("expected blocked refresh result");
        let ViewDataRefresh::Blocked(events) = outcome else {
            panic!("expected blocked refresh");
        };
        assert!(
            events
                .iter()
                .filter_map(|event| event.as_message())
                .any(|message| message.message.contains("Rendered view was not updated"))
        );

        let contents = fs::read_to_string(&path).expect("expected last good data");
        assert!(contents.contains("\\\\textrm{set}"));

        let _ = fs::remove_dir_all(dir);
    }
}
