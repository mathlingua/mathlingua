use crate::backend::collection::SourceCollection;
use crate::backend::view::CollectionView;
use crate::events::{Event, EventLog, EventLogListener};
use crate::mlg::util::{has_blocking_user_issues_since, no_errors_since};
use crate::mlg::view_assets::{ViewerPageConfig, configured_viewer_index, viewer_asset};
use serde_json::to_writer_pretty;
use std::fs;
use std::io;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc::{self, Receiver, Sender},
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

const ORIGIN: &str = "mlg_view";
const VIEW_BIND_HOST: &str = "0.0.0.0";
const COLLECTION_DATA_URL: &str = "/api/collection.json";
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

    let listener = bind_view_listener(port, event_log)?;
    let bound_port = listener.local_addr()?.port();
    let server = Server::from_listener(listener, None)
        .map_err(|error| io::Error::other(format!("Could not start viewer server: {error}")))?;
    let index = configured_viewer_index(&ViewerPageConfig {
        base_href: "/",
        route_base_path: "",
        collection_data_path: Some(COLLECTION_DATA_URL),
        static_data_base_path: None,
    })?;

    let view_session_dir = create_view_session_dir()?;
    let view_data_path = view_session_dir.join("collection.json");
    if let Err(error) = write_collection_view_data(&view_data_path, &collection_view) {
        let _ = fs::remove_dir_all(&view_session_dir);
        return Err(error);
    }

    let url = format!("http://localhost:{bound_port}/");
    event_log.user_log(Some(ORIGIN), format!("Starting viewer at {url}"));

    let (refresh_sender, refresh_receiver) = mpsc::channel();
    let stop_refresh = Arc::new(AtomicBool::new(false));
    let refresh_thread = spawn_view_data_refresher(
        cwd.to_path_buf(),
        view_data_path.clone(),
        Arc::clone(&stop_refresh),
        refresh_sender,
    );

    event_log.user_log(Some(ORIGIN), format!("Viewer ready at {url}"));
    let result = run_view_server(
        &server,
        &view_data_path,
        &index,
        &refresh_receiver,
        event_log,
    );
    stop_refresh.store(true, Ordering::Relaxed);
    join_view_data_refresher(refresh_thread, event_log);
    let _ = fs::remove_dir_all(&view_session_dir);
    result
}

fn bind_view_listener(port: u16, event_log: &mut EventLog) -> io::Result<TcpListener> {
    match TcpListener::bind((VIEW_BIND_HOST, port)) {
        Ok(listener) => Ok(listener),
        Err(error) => {
            let message = port_error_message(port, &error);
            event_log.user_error(Some(ORIGIN), message.clone());
            Err(io::Error::new(error.kind(), message))
        }
    }
}

#[cfg(test)]
fn check_port_is_available(port: u16, event_log: &mut EventLog) -> io::Result<()> {
    bind_view_listener(port, event_log).map(drop)
}

fn port_error_message(port: u16, error: &io::Error) -> String {
    match error.kind() {
        io::ErrorKind::AddrInUse => format!(
            "Port {port} is already in use by another program. \
             Stop that program, or start the viewer on a different port with `mlg view --port <PORT>`"
        ),
        io::ErrorKind::PermissionDenied => format!(
            "Port {port} cannot be opened because this user is not permitted to use it. \
             Ports below 1024 usually require elevated privileges, so try `mlg view --port <PORT>` with a port above 1024"
        ),
        _ => format!("Port {port} is not available for the viewer: {error}"),
    }
}

fn finish_view_setup_with_possible_errors(event_log: &mut EventLog) -> io::Result<()> {
    if event_log.has_errors() {
        Err(io::Error::other("Unable to start the viewer"))
    } else {
        event_log.user_log(Some(ORIGIN), "No Mathlingua files were found to render");
        Ok(())
    }
}

fn run_view_server(
    server: &Server,
    view_data_path: &Path,
    index: &[u8],
    refresh_receiver: &Receiver<Vec<Event>>,
    event_log: &mut EventLog,
) -> io::Result<()> {
    loop {
        drain_refresh_events(refresh_receiver, event_log);
        if let Some(request) = server.recv_timeout(Duration::from_millis(100))? {
            if let Err(error) = serve_view_request(request, view_data_path, index) {
                event_log.system_warning(
                    Some(ORIGIN),
                    format!("Could not complete a viewer request: {error}"),
                );
            }
        }
    }
}

fn serve_view_request(request: Request, view_data_path: &Path, index: &[u8]) -> io::Result<()> {
    if !matches!(request.method(), Method::Get | Method::Head) {
        return request
            .respond(Response::empty(StatusCode(405)).with_header(header("Allow", "GET, HEAD")?));
    }

    let url_path = request.url().split('?').next().unwrap_or("/");
    let asset_path = url_path.trim_start_matches('/');
    if asset_path.split('/').any(|part| part == "..") {
        return request.respond(Response::empty(StatusCode(400)));
    }

    if url_path == COLLECTION_DATA_URL {
        let body = fs::read(view_data_path)?;
        return request.respond(
            response(body, StatusCode(200), "application/json; charset=utf-8")?
                .with_header(header("Cache-Control", "no-store")?),
        );
    }

    if asset_path == "index.html" || asset_path.is_empty() {
        return request.respond(index_response(index)?);
    }

    if let Some(body) = viewer_asset(asset_path) {
        let mime = mime_guess::from_path(asset_path).first_or_octet_stream();
        return request.respond(
            response(body, StatusCode(200), mime.essence_str())?.with_header(header(
                "Cache-Control",
                "public, max-age=31536000, immutable",
            )?),
        );
    }

    if looks_like_asset_path(asset_path) {
        return request.respond(Response::empty(StatusCode(404)));
    }

    request.respond(index_response(index)?)
}

fn looks_like_asset_path(path: &str) -> bool {
    path.starts_with("assets/")
        || Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.contains('.'))
}

fn index_response(index: &[u8]) -> io::Result<Response<std::io::Cursor<Vec<u8>>>> {
    Ok(
        response(index.to_vec(), StatusCode(200), "text/html; charset=utf-8")?
            .with_header(header("Cache-Control", "no-cache")?),
    )
}

fn response(
    body: Vec<u8>,
    status: StatusCode,
    content_type: &str,
) -> io::Result<Response<std::io::Cursor<Vec<u8>>>> {
    Ok(Response::from_data(body)
        .with_status_code(status)
        .with_header(header("Content-Type", content_type)?))
}

fn header(name: &str, value: &str) -> io::Result<Header> {
    Header::from_bytes(name.as_bytes(), value.as_bytes())
        .map_err(|_| io::Error::other(format!("Invalid HTTP header `{name}: {value}`")))
}

fn drain_refresh_events(receiver: &Receiver<Vec<Event>>, event_log: &mut EventLog) {
    while let Ok(events) = receiver.try_recv() {
        for event in events {
            event_log.push(event);
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
        Ok(Err(error)) => event_log.system_warning(
            Some(ORIGIN),
            format!("Failed to refresh rendered view data: {error}"),
        ),
        Err(_) => {
            event_log.system_warning(Some(ORIGIN), "The rendered view refresher thread panicked")
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
        std::env::temp_dir().join(format!("mlg-view-{}-{}-{id}", std::process::id(), unique));
    fs::create_dir(&path)?;
    Ok(path)
}

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::{
        VIEW_BIND_HOST, ViewDataRefresh, check_port_is_available, create_view_session_dir,
        looks_like_asset_path, rebuild_collection_view_data, write_collection_view_data,
    };
    use crate::backend::view::{CollectionView, FileView, GroupView, PageView, SectionView};
    use crate::events::EventLog;
    use serde_json::Value;
    use std::fs;
    use std::net::TcpListener;

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
    fn distinguishes_viewer_routes_from_missing_assets() {
        assert!(!looks_like_asset_path("sets/groups"));
        assert!(looks_like_asset_path("assets/missing.js"));
        assert!(looks_like_asset_path("favicon.ico"));
    }

    #[test]
    fn writes_collection_view_data_as_json() {
        let dir = create_view_session_dir().expect("expected temp dir");
        let path = dir.join("collection.json");
        let collection = CollectionView {
            title: "demo".to_string(),
            preface: vec![],
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
                    parameter_destructurings: Vec::new(),
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
            "[\\set]\nDeclares: S\nDocumented:\n. called: \"set\"\n",
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
            "[\\set]\nDeclares: S\nDocumented:\n. called: \"updated set\"\n",
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
        fs::write(content.join("alpha_file.mlg"), "Title: \"Alpha\"\n").unwrap();
        fs::write(content.join("gamma_file.mlg"), "Title: \"Gamma\"\n").unwrap();
        fs::write(content.join("hidden_file.mlg"), "Title: \"Hidden\"\n").unwrap();
        fs::write(visible_dir.join("inside.mlg"), "Title: \"Inside\"\n").unwrap();
        fs::write(hidden_dir.join("inside.mlg"), "Title: \"Hidden Inside\"\n").unwrap();
        fs::write(
            content.join("toc"),
            "gamma_file.mlg -> Custom Gamma\nvisible_dir -> Visible Directory\nhidden_dir -> HIDDEN\nhidden_file.mlg -> HIDDEN\nalpha_file.mlg\n",
        )
        .unwrap();

        assert_eq!(
            rebuild_collection_view_data(&root, &path).expect("expected view data"),
            ViewDataRefresh::Updated
        );
        let contents = fs::read_to_string(&path).expect("expected collection data");
        let json: Value = serde_json::from_str(&contents).expect("expected json");
        let directories = json["directories"].as_array().unwrap();
        let files = json["files"].as_array().unwrap();
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
            "[\\set]\nDeclares: S\nDocumented:\n. called: \"set\"\n",
        )
        .unwrap();
        assert_eq!(
            rebuild_collection_view_data(&root, &path).unwrap(),
            ViewDataRefresh::Updated
        );

        fs::write(&file, "[\\set]\nDeclares: S\n").unwrap();
        let ViewDataRefresh::Blocked(events) = rebuild_collection_view_data(&root, &path).unwrap()
        else {
            panic!("expected blocked refresh");
        };
        assert!(
            events
                .iter()
                .filter_map(|event| event.as_message())
                .any(|message| { message.message.contains("Rendered view was not updated") })
        );
        let contents = fs::read_to_string(&path).expect("expected last good data");
        assert!(contents.contains("\\\\textrm{set}"));
        let _ = fs::remove_dir_all(dir);
    }
}
