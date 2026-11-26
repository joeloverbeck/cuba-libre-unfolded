use super::*;

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; drives the native proclamation composer"]
fn native_human_composes_a_proclamation_for_the_selected_faction() {
    let mut app = super::available_forces::start("broadsheet-compose-", "issue-230", "3840x2160");
    let pending = app.observe()["observation"].clone();
    let revision = pending["revision"].as_u64().unwrap();
    app.activate("proclamation:write", revision);
    let open = app.observe();
    assert_eq!(open["table"]["proclamation_composer"]["author"], "M26");
    assert_eq!(open["table"]["proclamation_composer"]["limit"], 400);
    assert_eq!(open["table"]["proclamation_composer"]["spent"], false);
    app.activate("proclamation:author:syndicate", revision);
    let chosen = app.observe();
    assert_eq!(
        chosen["table"]["proclamation_composer"]["author"],
        "Syndicate"
    );
    assert_eq!(chosen["observation"], pending);
    app.key("Escape");
    assert_eq!(app.observe()["observation"], pending);
    app.choose("pass");
    assert_ne!(app.revision(), revision);
    app.quit();
}
