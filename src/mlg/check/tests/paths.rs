use super::*;

#[test]
fn check_rejects_non_mlg_files_when_given_explicitly() {
    let temp_dir = TestDir::new();
    let file = temp_dir.path().join("notes.txt");

    fs::write(&file, "not mathlingua").unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(
        temp_dir.path(),
        &[PathBuf::from("notes.txt")],
        &mut event_log,
    );
    let user_events = user_events(&event_log);

    assert_eq!(result.files_checked, 0);
    assert_eq!(user_events.len(), 2);
    assert_eq!(
        user_events[0],
        Event::user_path_error(file.canonicalize().unwrap(), "Not a .mlg file")
            .with_origin("mlg_check")
    );
    assert_eq!(
        user_events[1],
        Event::user_log("Found 1 issue.").with_origin("mlg_check")
    );
}

#[test]
fn resolve_source_files_collects_explicit_files_and_directories() {
    let temp_dir = TestDir::new();
    let docs = temp_dir.path().join("docs");
    let nested = docs.join("nested");
    let extra = temp_dir.path().join("extra.mlg");

    fs::create_dir_all(&nested).unwrap();
    fs::write(docs.join("a.mlg"), "Defines: A\n").unwrap();
    fs::write(nested.join("b.mlg"), "Defines: B\n").unwrap();
    fs::write(&extra, "Defines: C\n").unwrap();

    let mut event_log = EventLog::new();
    let files = resolve_source_files(
        temp_dir.path(),
        &[PathBuf::from("docs"), PathBuf::from("extra.mlg")],
        &mut event_log,
        "mlg_check",
    );

    assert!(user_events(&event_log).is_empty());
    assert_eq!(files.len(), 3);
}

#[test]
fn finds_collection_root_in_ancestor_directories() {
    let temp_dir = TestDir::new();
    let root = temp_dir.path().join("repo");
    let nested = root.join("content/logic");

    fs::create_dir_all(&nested).unwrap();
    fs::write(root.join("mlg.json"), default_config_contents()).unwrap();

    let discovered = find_collection_root(&nested).expect("expected collection root");

    assert_eq!(discovered, root);
}
