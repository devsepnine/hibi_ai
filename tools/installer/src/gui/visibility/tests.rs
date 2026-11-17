use super::actions::Action;
use crate::app::View;
use crate::tree::test_support::make_component;
use std::collections::BTreeSet;
#[path = "filters/tests.rs"]
mod filters;
#[path = "test_support.rs"]
mod test_support;
use test_support::Desktop;

#[test]
fn unhide_button_applies_one_target_state_to_mixed_selection_and_repeated_clicks() {
    let mut desktop = Desktop::new();
    desktop.click("Unhide selected · 3");
    assert!(desktop.app().hidden_folders.is_empty());
    assert!(desktop.saved().is_empty());
    assert!(
        desktop.app().components[..3]
            .iter()
            .all(|item| item.selected)
    );
    desktop.click("Unhide selected · 3");
    assert!(desktop.saved().is_empty());
    assert_eq!(desktop.app().current_view, View::List);
}

#[test]
fn hide_button_keeps_already_hidden_folders_hidden_and_clears_the_selection() {
    let mut desktop = Desktop::new();
    desktop.click("Hide selected · 3");
    assert_eq!(desktop.app().current_view, View::ConfirmHide);
    assert_eq!(
        desktop
            .app()
            .hide_confirmation
            .as_ref()
            .unwrap()
            .folders
            .len(),
        3
    );
    desktop.click("Hide folders");
    assert_eq!(
        desktop.saved(),
        BTreeSet::from([
            desktop.app().dest_dir.join("skills/visible"),
            desktop.app().dest_dir.join("skills/hidden"),
            desktop.app().dest_dir.join("skills/other-hidden"),
        ])
    );
    assert!(desktop.app().components.iter().all(|item| !item.selected));
    desktop.click("Hide selected · 0");
    desktop.click("Unhide selected · 0");
    assert_eq!(desktop.app().current_view, View::List);
    assert_eq!(desktop.saved().len(), 3);
}

#[test]
fn bulk_unhide_save_failure_preserves_settings_and_checks() {
    let mut desktop = Desktop::new();
    let before = desktop.app().hidden_folders.clone();
    std::fs::write(&desktop.app().visibility_path, "invalid JSON").unwrap();
    desktop.click("Unhide selected · 3");
    assert_eq!(desktop.app().hidden_folders, before);
    assert!(
        desktop.app().components[..3]
            .iter()
            .all(|item| item.selected)
    );
    assert_eq!(
        std::fs::read_to_string(&desktop.app().visibility_path).unwrap(),
        "invalid JSON"
    );
    assert!(
        desktop
            .app()
            .status_message
            .as_deref()
            .unwrap()
            .starts_with("Failed to save folder visibility:")
    );
}

#[test]
fn late_visibility_actions_do_not_change_settings_behind_a_confirmation() {
    let mut desktop = Desktop::new();
    desktop.state.dispatch(Action::RequestClose);
    assert!(desktop.state.close_requested);
    let before = desktop.saved();
    desktop.state.dispatch(Action::HideSelectedFolders);
    desktop.state.dispatch(Action::UnhideSelectedFolders);
    assert_eq!(desktop.saved(), before);
    assert_eq!(desktop.app().current_view, View::List);
}

#[test]
fn repeated_hide_clicks_never_toggle_already_hidden_folders_visible() {
    let mut desktop = Desktop::new();
    for _ in 0..2 {
        desktop.click("Select all");
        desktop.click("Hide selected · 4");
        desktop.click("Hide folders");
        assert_eq!(desktop.saved().len(), 4);
        assert!(
            desktop
                .app()
                .components
                .iter()
                .all(|item| desktop.app().is_component_hidden(item))
        );
    }
}

#[test]
fn unhiding_a_selected_parent_clears_nested_rules_and_preserves_siblings() {
    let mut desktop = Desktop::new();
    let app = desktop.state.app.as_mut().unwrap();
    app.components = ["parent/one/a.md", "parent/two/a.md", "parent-other/a.md"]
        .map(make_component)
        .into();
    app.components[2].selected = false;
    let hidden = BTreeSet::from([
        app.dest_dir.join("skills/parent"),
        app.dest_dir.join("skills/parent/one"),
        app.dest_dir.join("skills/parent-other"),
    ]);
    crate::fs::installer::visibility::save(&app.visibility_path, &app.hidden_folders, &hidden)
        .unwrap();
    app.finish_loading(app.components.clone(), vec![], vec![], vec![]);
    desktop.click("Unhide selected · 1");
    assert_eq!(
        desktop.saved(),
        BTreeSet::from([desktop.app().dest_dir.join("skills/parent-other")])
    );
}

#[test]
fn unhide_does_not_silently_reveal_an_unselected_hidden_parent() {
    let mut desktop = Desktop::new();
    let app = desktop.state.app.as_mut().unwrap();
    app.components = ["parent/one/a.md", "parent/two/a.md"]
        .map(make_component)
        .into();
    app.components[1].selected = false;
    let hidden = BTreeSet::from([app.dest_dir.join("skills/parent")]);
    crate::fs::installer::visibility::save(&app.visibility_path, &app.hidden_folders, &hidden)
        .unwrap();
    app.finish_loading(app.components.clone(), vec![], vec![], vec![]);
    let before = desktop.saved();
    desktop.click("Unhide selected · 1");
    assert_eq!(desktop.saved(), before);
    assert!(
        desktop
            .app()
            .status_message
            .as_deref()
            .unwrap()
            .contains("Select that parent folder")
    );
}
