#[test]
fn check_result_markers_bound_the_check_events() {
    let temp_dir = TestDir::new();
    let root = temp_dir.path().join("repo");

    fs::create_dir_all(root.join("content")).unwrap();
    fs::write(root.join("mlg.json"), default_config_contents()).unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(&root, &[], &mut event_log);
    let range_events = event_log
        .events_between(&result.marker_range.begin, &result.marker_range.end)
        .expect("expected event range");

    assert!(
        range_events
            .iter()
            .filter_map(|event| event.as_message())
            .any(|event| event.level == Level::Log && event.message == "Checked 0 files")
    );
}
