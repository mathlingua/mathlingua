#[test]
fn check_without_arguments_uses_collection_content_from_a_nested_directory() {
    let temp_dir = TestDir::new();
    let root = temp_dir.path().join("repo");
    let nested_cwd = root.join("content/algebra");

    fs::create_dir_all(&nested_cwd).unwrap();
    fs::write(root.join("mlg.json"), default_config_contents()).unwrap();
    fs::write(root.join("content/sets.mlg"), "Title: \"Sets\"\n").unwrap();
    fs::write(nested_cwd.join("groups.mlg"), "Title: \"Groups\"\n").unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(&nested_cwd, &[], &mut event_log);

    assert_eq!(result.files_checked, 2);
    assert_eq!(
        user_events(&event_log),
        [Event::user_log("Checked 2 files").with_origin("mlg_check")]
    );
    assert!(
        event_log
            .events_between(&result.marker_range.begin, &result.marker_range.end)
            .is_some()
    );
}

#[test]
fn check_without_arguments_errors_when_not_in_a_collection() {
    let temp_dir = TestDir::new();

    let mut event_log = EventLog::new();
    let result = check_in(temp_dir.path(), &[], &mut event_log);
    let user_events = user_events(&event_log);

    assert_eq!(result.files_checked, 0);
    assert_eq!(event_log.issue_count(), 1);
    assert_eq!(
        user_events[0].as_message().unwrap().message,
        "Not inside a Mathlingua collection and no paths were provided"
    );
    assert_eq!(
        user_events[1],
        Event::user_log("Found 1 issue.").with_origin("mlg_check")
    );
    assert!(event_log.has_errors());
}

#[test]
fn check_with_directory_argument_processes_mlg_files_recursively() {
    let temp_dir = TestDir::new();
    let docs = temp_dir.path().join("docs/logic");

    fs::create_dir_all(&docs).unwrap();
    fs::write(docs.join("intro.mlg"), "Title: \"Intro\"\n").unwrap();
    fs::write(docs.join("notes.txt"), "ignore me").unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(temp_dir.path(), &[PathBuf::from("docs")], &mut event_log);

    assert_eq!(result.files_checked, 1);
    assert_eq!(
        user_events(&event_log),
        [Event::user_log("Checked 1 file").with_origin("mlg_check")]
    );
}

#[test]
fn check_with_empty_content_directory_succeeds() {
    let temp_dir = TestDir::new();
    let root = temp_dir.path().join("repo");

    fs::create_dir_all(root.join("content")).unwrap();
    fs::write(root.join("mlg.json"), default_config_contents()).unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(&root, &[], &mut event_log);

    assert_eq!(result.files_checked, 0);
    assert_eq!(
        user_events(&event_log),
        [Event::user_log("Checked 0 files").with_origin("mlg_check")]
    );
}

#[test]
fn check_reports_config_validation_errors() {
    let temp_dir = TestDir::new();
    let root = temp_dir.path().join("repo");
    fs::create_dir_all(root.join("content")).unwrap();
    fs::write(root.join("mlg.json"), r#"{"name": 5}"#).unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(&root, &[], &mut event_log);

    assert_eq!(result.files_checked, 0);
    let messages: Vec<&str> = event_log
        .events()
        .iter()
        .filter_map(Event::as_message)
        .filter(|message| message.audience == Audience::User && message.level == Level::Error)
        .map(|message| message.message.as_str())
        .collect();
    assert_eq!(
        messages,
        vec![
            "mlg.json field \"name\" must be a string",
            "mlg.json is missing required field \"version\"",
        ]
    );
    assert!(event_log.has_errors());
}

#[test]
fn check_reports_proto_events_for_invalid_files() {
    let temp_dir = TestDir::new();
    let file = temp_dir.path().join("broken.mlg");

    fs::write(&file, "Defines: 'f(x_)'\n").unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(
        temp_dir.path(),
        &[PathBuf::from("broken.mlg")],
        &mut event_log,
    );
    let events = event_log.events();

    assert_eq!(result.files_checked, 1);
    assert!(events.iter().filter_map(Event::as_message).any(|event| {
        event.location.as_ref().is_some_and(|location| {
            matches!(
                location,
                crate::events::EventLocation::File { path, .. }
                    if *path == file.canonicalize().unwrap()
            )
        }) && event.message == "Single-quoted formulations are not allowed"
    }));
    assert_eq!(
        user_events(&event_log)
            .last()
            .cloned()
            .expect("expected summary event"),
        Event::user_log("Found 2 issues.").with_origin("mlg_check")
    );
}

