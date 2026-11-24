use super::*;

const MANAGED_STATUSES: [InstallStatus; 4] = [
    InstallStatus::New,
    InstallStatus::Modified,
    InstallStatus::Unchanged,
    InstallStatus::Managed,
];

fn with_status(status: InstallStatus) -> (Fixture, App) {
    let (fixture, mut app) = fixture();
    app.components[0].status = status;
    (fixture, app)
}

#[test]
fn each_managed_status_can_be_hidden_persisted_and_unhidden_without_changing_files() {
    for status in MANAGED_STATUSES {
        let (_fixture, mut app) = with_status(status);
        let file = app.dest_dir.join("skills/.system/tool/SKILL.md");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "managed contents").unwrap();
        hide(&mut app);
        assert!(!screen(&app).contains(".system/"));
        assert!(!app.components[0].selected);
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "managed contents");
        assert_eq!(
            crate::fs::installer::visibility::load(&app.visibility_path).unwrap(),
            std::collections::BTreeSet::from([app.dest_dir.join("skills/.system")])
        );
        app.finish_loading(app.components.clone(), vec![], vec![], vec![]);
        assert!(!screen(&app).contains(".system/"));
        press(&mut app, 'H');
        focus_folder(&mut app, ".system");
        press(&mut app, 'x');
        assert!(app.hidden_folders.is_empty());
        assert!(screen(&app).contains(".system/"));
        assert!(!app.components[0].selected);
    }
}

#[test]
fn hidden_managed_items_are_excluded_from_bulk_selection_and_both_work_queues() {
    for status in MANAGED_STATUSES {
        let (_fixture, mut app) = with_status(status);
        hide(&mut app);
        press(&mut app, 'a');
        press(&mut app, 'a');
        assert!(!app.components[0].selected);
        assert!(app.components[1].selected);
        app.components[0].selected = true;
        for command in ['i', 'r'] {
            press(&mut app, command);
            if command == 'r' {
                assert_eq!(app.current_view, View::ConfirmRemove(1));
                press(&mut app, 'y');
            }
            assert_eq!(app.processing_queue, [1]);
            app.close_processing();
        }
        press(&mut app, 'H');
        press(&mut app, 'i');
        assert_eq!(app.processing_queue, [0, 1]);
    }
}

#[test]
fn cancelling_or_failing_to_save_managed_hide_preserves_checks_and_visibility() {
    for status in MANAGED_STATUSES {
        let (_fixture, mut app) = with_status(status);
        press(&mut app, 'x');
        assert_eq!(app.current_view, View::ConfirmHide);
        press(&mut app, 'n');
        assert!(app.components[0].selected);
        assert!(app.hidden_folders.is_empty());
        assert!(!app.visibility_path.exists());
        let invalid_path = app.visibility_path.parent().unwrap().join("blocker");
        std::fs::write(&invalid_path, "preserve this file").unwrap();
        app.visibility_path = invalid_path.join("visibility.json");
        hide(&mut app);
        assert!(app.components[0].selected);
        assert!(app.hidden_folders.is_empty());
        assert!(screen(&app).contains(".system/"));
        assert!(
            app.status_message
                .as_deref()
                .unwrap()
                .starts_with("Failed to save folder visibility:")
        );
        assert_eq!(
            std::fs::read_to_string(invalid_path).unwrap(),
            "preserve this file"
        );
    }
}

#[test]
fn status_changes_between_request_and_confirmation_do_not_cancel_batch_hide() {
    let (_fixture, mut app) = fixture();
    press(&mut app, 'X');
    assert_eq!(app.current_view, View::ConfirmHide);
    app.components[0].status = InstallStatus::Modified;
    press(&mut app, 'y');
    assert_eq!(app.hidden_folder_count(), 2);
    assert!(!app.components[0].selected);
    assert!(!app.components[1].selected);
    assert!(app.visible_components().is_empty());
}
