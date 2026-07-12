use crate::backend::collection::{DOCS_DIR, SourceCollection};
use crate::backend::view::{CollectionView, DirectoryView, GroupView};
use crate::events::{EventLog, EventLogListener};
use crate::mlg::util::{has_blocking_user_issues_since, no_errors_since};
use serde::Serialize;
use serde_json::to_writer_pretty;
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const NEXTJS_ORIGIN: &str = "nextjs";
const ORIGIN: &str = "mlg_export";
const DATA_DIR: &str = "data";
const MANIFEST_FILE: &str = "manifest.json";

pub struct ExportResult {
    pub event_log: EventLog,
    pub successful: bool,
}

pub fn export(
    cwd: &Path,
    base_path: Option<&str>,
    cname: Option<&str>,
    force: bool,
    listener: Option<Box<dyn EventLogListener>>,
) -> ExportResult {
    let mut event_log = EventLog::new();
    if let Some(listener) = listener {
        event_log.add_boxed_listener(listener);
    }

    let starting_event_count = event_log.events().len();
    let io_ok = export_in(cwd, base_path, cname, force, &mut event_log).is_ok();
    let successful = io_ok && no_errors_since(&event_log, starting_event_count);

    ExportResult {
        event_log,
        successful,
    }
}

pub(super) fn export_in(
    cwd: &Path,
    base_path: Option<&str>,
    cname: Option<&str>,
    force: bool,
    event_log: &mut EventLog,
) -> io::Result<()> {
    let export_options = match ExportOptions::new(base_path, cname) {
        Ok(options) => options,
        Err(message) => {
            event_log.user_error(Some(ORIGIN), message.clone());
            return Err(io::Error::other(message));
        }
    };
    let starting_event_count = event_log.events().len();
    let mut collection = SourceCollection::load(cwd, event_log, ORIGIN);
    // The static site always builds into `<collection root>/docs`, a sibling of
    // `content/` and `metadata/` (the conventional GitHub Pages source folder).
    let output = collection.root().join(DOCS_DIR);
    if collection.source_files().is_empty() {
        event_log.user_error(Some(ORIGIN), "No Mathlingua files were found to export");
        return Err(io::Error::other("No Mathlingua files were found to export"));
    }

    event_log.system_debug(
        Some(ORIGIN),
        format!(
            "Checking collection before exporting {} file(s)",
            collection.source_files().len()
        ),
    );

    collection.run_check_passes(event_log, ORIGIN);

    if has_blocking_user_issues_since(event_log, starting_event_count) {
        event_log.user_error(
            Some(ORIGIN),
            "Export stopped because one or more files could not be rendered",
        );
        return Err(io::Error::other(
            "One or more files could not be rendered for export",
        ));
    }

    let Some(collection_view) = collection.build_view(event_log) else {
        event_log.user_error(
            Some(ORIGIN),
            "Export stopped because one or more files could not be rendered",
        );
        return Err(io::Error::other(
            "One or more files could not be rendered for export",
        ));
    };

    prepare_output_directory(&output, force, event_log)?;

    let temp_dir = create_export_session_dir()?;
    let data_dir = temp_dir.join(DATA_DIR);
    write_static_export_data(&data_dir, &collection_view)?;

    ensure_web_dependencies(event_log)?;
    build_static_web_app(&data_dir, &export_options, event_log)?;
    copy_static_web_app(&output)?;
    copy_dir_contents(&data_dir, &output.join(DATA_DIR))?;
    write_github_pages_files(&output, &export_options)?;

    let _ = fs::remove_dir_all(&temp_dir);
    event_log.user_log(
        Some(ORIGIN),
        format!("Exported static site to {}", output.display()),
    );
    Ok(())
}

fn prepare_output_directory(
    output: &Path,
    force: bool,
    event_log: &mut EventLog,
) -> io::Result<()> {
    if output.exists() {
        if output.is_file() {
            event_log.user_error_at_path(
                Some(ORIGIN),
                output,
                "Export output path exists and is a file",
            );
            return Err(io::Error::other("Export output path exists and is a file"));
        }

        if !force && !is_empty_directory(output)? {
            event_log.user_error_at_path(
                Some(ORIGIN),
                output,
                "Export output directory is not empty; pass --force to replace it",
            );
            return Err(io::Error::other("Export output directory is not empty"));
        }

        fs::remove_dir_all(output)?;
    }

    fs::create_dir_all(output)
}

fn is_empty_directory(path: &Path) -> io::Result<bool> {
    Ok(fs::read_dir(path)?.next().is_none())
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
    )
}

fn build_static_web_app(
    data_dir: &Path,
    options: &ExportOptions,
    event_log: &mut EventLog,
) -> io::Result<()> {
    event_log.user_log(Some(ORIGIN), "Building static viewer");

    let mut command = Command::new("npm");
    command
        .arg("run")
        .arg("build")
        .current_dir(web_app_directory())
        .env("MLG_STATIC_EXPORT", "1")
        .env("MLG_EXPORT_DATA_DIR", data_dir)
        .env("MLG_EXPORT_BASE_PATH", &options.base_path)
        .env("NEXT_TELEMETRY_DISABLED", "1");

    run_child(command, NEXTJS_ORIGIN, event_log)
}

fn run_child(mut command: Command, origin: &str, event_log: &mut EventLog) -> io::Result<()> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let output = match command.output() {
        Ok(output) => output,
        Err(error) => {
            event_log.user_error(
                Some(ORIGIN),
                format!("Failed to start the web export process: {error}"),
            );
            return Err(error);
        }
    };

    log_process_output(&output.stdout, origin, event_log);
    log_process_output(&output.stderr, origin, event_log);

    if output.status.success() {
        Ok(())
    } else {
        event_log.user_error(
            Some(ORIGIN),
            format!(
                "The web export process exited with status {}",
                output.status
            ),
        );
        Err(io::Error::other(format!(
            "The web export process exited with status {}",
            output.status
        )))
    }
}

fn log_process_output(output: &[u8], origin: &str, event_log: &mut EventLog) {
    for line in String::from_utf8_lossy(output).lines() {
        event_log.system_log(Some(origin), line.to_string());
    }
}

fn copy_static_web_app(output: &Path) -> io::Result<()> {
    copy_dir_contents(&web_app_directory().join("out"), output)
}

fn write_github_pages_files(output: &Path, options: &ExportOptions) -> io::Result<()> {
    fs::write(output.join(".nojekyll"), "")?;
    write_cname_file(output, options)
}

fn write_cname_file(output: &Path, options: &ExportOptions) -> io::Result<()> {
    let Some(cname) = options.cname.as_deref() else {
        return Ok(());
    };

    fs::write(output.join("CNAME"), format!("{cname}\n"))
}

fn copy_dir_contents(source: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());

        if entry.file_type()?.is_dir() {
            copy_dir_contents(&source_path, &destination_path)?;
        } else {
            fs::copy(&source_path, &destination_path)?;
        }
    }

    Ok(())
}

fn write_static_export_data(data_dir: &Path, collection: &CollectionView) -> io::Result<()> {
    fs::create_dir_all(data_dir.join("pages"))?;
    fs::create_dir_all(data_dir.join("items"))?;

    let mut manifest = ExportManifest {
        schema_version: 1,
        title: collection.title.clone(),
        directories: collection.directories.clone(),
        files: Vec::new(),
        definitions: BTreeMap::new(),
        items: BTreeMap::new(),
    };

    for file in &collection.files {
        let page_data_path = page_data_path(&file.path);
        let mut page = ExportPage {
            path: file.path.clone(),
            title: file.title.clone(),
            item_ids: Vec::new(),
        };

        for (index, item) in file.items.iter().enumerate() {
            let item_id = export_item_id(item, &file.path, index);
            let item_path = item_data_path(&item_id);

            for key in &item.definition_keys {
                manifest
                    .definitions
                    .entry(key.clone())
                    .or_insert_with(|| item_id.clone());
            }

            manifest
                .items
                .entry(item_id.clone())
                .or_insert(item_path.clone());
            page.item_ids.push(item_id);
            write_json(data_dir.join(&item_path), item)?;
        }

        write_json(data_dir.join(&page_data_path), &page)?;
        manifest.files.push(ExportFile {
            path: file.path.clone(),
            title: file.title.clone(),
            data_path: page_data_path,
        });
    }

    write_json(data_dir.join(MANIFEST_FILE), &manifest)
}

fn write_json(path: PathBuf, value: &impl Serialize) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = fs::File::create(path)?;
    to_writer_pretty(file, value)
        .map_err(|error| io::Error::other(format!("Failed to write export JSON: {error}")))
}

fn export_item_id(item: &GroupView, file_path: &str, index: usize) -> String {
    if item.id.trim().is_empty() {
        format!("{}#{index}", route_path(file_path))
    } else {
        item.id.clone()
    }
}

fn page_data_path(file_path: &str) -> String {
    format!("pages/{}.json", route_path(file_path))
}

fn item_data_path(item_id: &str) -> String {
    format!("items/{}.json", sanitize_data_file_stem(item_id))
}

fn route_path(file_path: &str) -> String {
    let normalized = file_path.replace('\\', "/");
    let content_relative = normalized.strip_prefix("content/").unwrap_or(&normalized);
    let without_extension = content_relative
        .strip_suffix(".mlg")
        .unwrap_or(content_relative);
    normalize_route_path(without_extension)
}

fn normalize_route_path(path: &str) -> String {
    let normalized = path
        .split('/')
        .filter(|segment| !segment.trim().is_empty())
        .map(|segment| segment.trim().replace(char::is_whitespace, "_"))
        .collect::<Vec<_>>()
        .join("/");

    if normalized.is_empty() {
        "index".to_string()
    } else {
        normalized
    }
}

fn sanitize_data_file_stem(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    if sanitized.is_empty() {
        "item".to_string()
    } else {
        sanitized
    }
}

fn create_export_session_dir() -> io::Result<PathBuf> {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("mlg-export-{}-{unique}", std::process::id()));
    fs::create_dir(&path)?;
    Ok(path)
}

fn web_app_directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("web")
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExportOptions {
    base_path: String,
    cname: Option<String>,
}

impl ExportOptions {
    fn new(base_path: Option<&str>, cname: Option<&str>) -> Result<Self, String> {
        Ok(Self {
            base_path: normalize_base_path(base_path)?,
            cname: normalize_cname(cname)?,
        })
    }
}

fn normalize_base_path(value: Option<&str>) -> Result<String, String> {
    let Some(value) = value else {
        return Ok(String::new());
    };
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "/" {
        return Ok(String::new());
    }
    if trimmed.contains("://") {
        return Err("Export base path must be a path such as `/repo-name`, not a URL".to_string());
    }
    if trimmed.contains('?') || trimmed.contains('#') {
        return Err("Export base path cannot contain query strings or fragments".to_string());
    }

    let path = if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    };

    Ok(path.trim_end_matches('/').to_string())
}

fn normalize_cname(value: Option<&str>) -> Result<Option<String>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    let trimmed = value.trim().trim_end_matches('.');
    if trimmed.is_empty() {
        return Err("CNAME domain cannot be empty".to_string());
    }
    if trimmed.contains("://") || trimmed.contains('/') {
        return Err("CNAME must be a domain name, not a URL".to_string());
    }

    Ok(Some(trimmed.to_string()))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportManifest {
    schema_version: u32,
    title: String,
    directories: Vec<DirectoryView>,
    files: Vec<ExportFile>,
    definitions: BTreeMap<String, String>,
    items: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportFile {
    path: String,
    title: Option<String>,
    data_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportPage {
    path: String,
    title: Option<String>,
    item_ids: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::{
        ExportOptions, MANIFEST_FILE, normalize_base_path, normalize_cname,
        write_github_pages_files, write_static_export_data,
    };
    use crate::backend::view::{CollectionView, FileView, GroupView, PageView};
    use serde::Deserialize;
    use serde::de::DeserializeOwned;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn writes_sharded_static_export_data() {
        let temp_dir = TestDir::new();
        let data_dir = temp_dir.path().join("data");
        let collection = CollectionView {
            title: "Demo".to_string(),
            directories: Vec::new(),
            files: vec![FileView {
                path: "content/sets/intro.mlg".to_string(),
                title: Some("Intro".to_string()),
                items: vec![
                    GroupView {
                        id: "18582990-701a-40d3-8ce3-ae12bd08a561".to_string(),
                        kind: "Title".to_string(),
                        definition_keys: Vec::new(),
                        heading: None,
                        heading_latex: None,
                        body_text: None,
                        page: Some(PageView {
                            kind: "Title".to_string(),
                            text: "Intro".to_string(),
                        }),
                        source: "Title: \"Intro\"".to_string(),
                        sections: Vec::new(),
                    },
                    GroupView {
                        id: "059126b9-dc83-41a2-aa1c-84f8e942f8d6".to_string(),
                        kind: "Describes".to_string(),
                        definition_keys: vec!["\\set".to_string()],
                        heading: Some("\\set".to_string()),
                        heading_latex: Some("\\textrm{set}".to_string()),
                        body_text: None,
                        page: None,
                        source: "[\\set]".to_string(),
                        sections: Vec::new(),
                    },
                ],
            }],
        };

        write_static_export_data(&data_dir, &collection).expect("expected export data");

        let manifest: ManifestFixture = read_json(&data_dir.join(MANIFEST_FILE));
        assert_eq!(manifest.title, "Demo");
        assert_eq!(manifest.files[0].data_path, "pages/sets/intro.json");
        assert_eq!(
            manifest.definitions.get("\\set"),
            Some(&"059126b9-dc83-41a2-aa1c-84f8e942f8d6".to_string())
        );
        assert_eq!(
            manifest.items.get("059126b9-dc83-41a2-aa1c-84f8e942f8d6"),
            Some(&"items/059126b9-dc83-41a2-aa1c-84f8e942f8d6.json".to_string())
        );

        let page: PageFixture = read_json(&data_dir.join("pages/sets/intro.json"));
        assert_eq!(
            page.item_ids,
            vec![
                "18582990-701a-40d3-8ce3-ae12bd08a561".to_string(),
                "059126b9-dc83-41a2-aa1c-84f8e942f8d6".to_string(),
            ]
        );
        assert!(
            data_dir
                .join("items/059126b9-dc83-41a2-aa1c-84f8e942f8d6.json")
                .is_file()
        );
    }

    #[test]
    fn normalizes_github_pages_base_paths() {
        assert_eq!(normalize_base_path(None).unwrap(), "");
        assert_eq!(normalize_base_path(Some("")).unwrap(), "");
        assert_eq!(normalize_base_path(Some("/")).unwrap(), "");
        assert_eq!(normalize_base_path(Some("mathlore")).unwrap(), "/mathlore");
        assert_eq!(
            normalize_base_path(Some("/mathlore/")).unwrap(),
            "/mathlore"
        );
        assert!(normalize_base_path(Some("https://example.org/mathlore")).is_err());
    }

    #[test]
    fn normalizes_cname_values() {
        assert_eq!(normalize_cname(None).unwrap(), None);
        assert_eq!(
            normalize_cname(Some("math.example.org.")).unwrap(),
            Some("math.example.org".to_string())
        );
        assert!(normalize_cname(Some("https://math.example.org")).is_err());
        assert!(normalize_cname(Some("")).is_err());
    }

    #[test]
    fn writes_github_pages_support_files() {
        let temp_dir = TestDir::new();
        let options = ExportOptions {
            base_path: String::new(),
            cname: Some("math.example.org".to_string()),
        };

        write_github_pages_files(temp_dir.path(), &options).expect("expected support files");

        assert_eq!(
            fs::read_to_string(temp_dir.path().join("CNAME")).unwrap(),
            "math.example.org\n"
        );
        assert_eq!(
            fs::read_to_string(temp_dir.path().join(".nojekyll")).unwrap(),
            ""
        );
    }

    fn read_json<T: DeserializeOwned>(path: &Path) -> T {
        serde_json::from_str(&fs::read_to_string(path).expect("expected json"))
            .expect("expected parsed json")
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ManifestFixture {
        title: String,
        files: Vec<FileFixture>,
        definitions: std::collections::BTreeMap<String, String>,
        items: std::collections::BTreeMap<String, String>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct FileFixture {
        data_path: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct PageFixture {
        item_ids: Vec<String>,
    }

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let sequence = COUNTER.fetch_add(1, Ordering::Relaxed);
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "mlg-export-test-{}-{unique}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("expected temp dir");
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
