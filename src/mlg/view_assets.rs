use rust_embed::RustEmbed;
use serde_json::{Map, Value};
use std::fs;
use std::io;
use std::path::Path;

const BASE_HREF_MARKER: &str = "__MLG_BASE_HREF__";
const VIEW_CONFIG_MARKER: &str = "__MLG_RUNTIME_CONFIG_JSON__";

#[derive(RustEmbed)]
#[folder = "web/dist/"]
struct ViewerAssets;

pub(super) struct ViewerPageConfig<'a> {
    pub base_href: &'a str,
    pub route_base_path: &'a str,
    pub collection_data_path: Option<&'a str>,
    pub static_data_base_path: Option<&'a str>,
}

pub(super) fn viewer_asset(path: &str) -> Option<Vec<u8>> {
    ViewerAssets::get(path).map(|file| file.data.into_owned())
}

pub(super) fn configured_viewer_index(config: &ViewerPageConfig<'_>) -> io::Result<Vec<u8>> {
    let template = ViewerAssets::get("index.html")
        .ok_or_else(|| io::Error::other("Embedded viewer index is missing"))?;
    let template = std::str::from_utf8(template.data.as_ref()).map_err(|error| {
        io::Error::other(format!("Embedded viewer index is not UTF-8: {error}"))
    })?;

    if !template.contains(BASE_HREF_MARKER) || !template.contains(VIEW_CONFIG_MARKER) {
        return Err(io::Error::other(
            "Embedded viewer index is missing its runtime configuration markers",
        ));
    }

    let mut runtime = Map::new();
    if !config.route_base_path.is_empty() {
        runtime.insert(
            "routeBasePath".to_owned(),
            Value::String(config.route_base_path.to_owned()),
        );
    }
    if let Some(path) = config.collection_data_path {
        runtime.insert(
            "collectionDataPath".to_owned(),
            Value::String(path.to_owned()),
        );
    }
    if let Some(path) = config.static_data_base_path {
        runtime.insert(
            "staticDataBasePath".to_owned(),
            Value::String(path.to_owned()),
        );
    }

    let runtime = serde_json::to_string(&runtime).map_err(|error| {
        io::Error::other(format!("Could not encode viewer configuration: {error}"))
    })?;
    let index = template
        .replace(BASE_HREF_MARKER, &escape_html_attribute(config.base_href))
        .replace(VIEW_CONFIG_MARKER, &runtime);
    Ok(index.into_bytes())
}

pub(super) fn copy_embedded_viewer(destination: &Path, index: &[u8]) -> io::Result<()> {
    for path in ViewerAssets::iter() {
        let path = path.as_ref();
        if path == "index.html" {
            continue;
        }
        let Some(file) = ViewerAssets::get(path) else {
            continue;
        };
        let destination_path = destination.join(path);
        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(destination_path, file.data.as_ref())?;
    }

    fs::create_dir_all(destination)?;
    fs::write(destination.join("index.html"), index)
}

fn escape_html_attribute(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::{
        BASE_HREF_MARKER, VIEW_CONFIG_MARKER, ViewerPageConfig, configured_viewer_index,
        viewer_asset,
    };
    #[test]
    fn configures_the_embedded_viewer_index() {
        let index = configured_viewer_index(&ViewerPageConfig {
            base_href: "/demo/",
            route_base_path: "/demo",
            collection_data_path: None,
            static_data_base_path: Some("/demo/data"),
        })
        .expect("expected configured viewer index");
        let index = String::from_utf8(index).expect("expected UTF-8 viewer index");

        assert!(index.contains(r#"<base href="/demo/""#));
        assert!(index.contains(r#""routeBasePath":"/demo""#));
        assert!(index.contains(r#""staticDataBasePath":"/demo/data""#));
        assert!(!index.contains(BASE_HREF_MARKER));
        assert!(!index.contains(VIEW_CONFIG_MARKER));
    }

    #[test]
    fn embeds_the_viewer_javascript() {
        assert!(viewer_asset("index.html").is_some());
        assert!(
            super::ViewerAssets::iter()
                .any(|path| path.starts_with("assets/") && path.ends_with(".js"))
        );
    }
}
