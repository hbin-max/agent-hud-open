use super::*;
use std::fs;

fn install(directory: &Path) {
    fs::create_dir_all(directory).unwrap();
    for name in [
        "Agent HUD.exe",
        "uninstall.exe",
        "agent-hud-engine.exe",
        "agent-hud-mcp.exe",
    ] {
        fs::write(directory.join(name), b"test fixture").unwrap();
    }
}

fn check(directory: &Path, current: &Path) -> Option<PathBuf> {
    let quoted = |path: &Path| format!("\"{}\"", path.display());
    approved(
        &quoted(directory),
        &quoted(&directory.join("Agent HUD.exe")),
        &quoted(&directory.join("uninstall.exe")),
        current,
    )
}

#[test]
fn registered_custom_unicode_installation_is_approved() {
    let temp = tempfile::tempdir().unwrap();
    let directory = temp.path().join("用户自选目录 Agent HUD");
    install(&directory);
    let executable = directory.join("Agent HUD.exe");
    assert_eq!(check(&directory, &executable), Some(executable));
}

#[test]
fn copied_executable_stale_registration_and_partial_install_are_refused() {
    let temp = tempfile::tempdir().unwrap();
    let directory = temp.path().join("installed");
    install(&directory);
    let executable = directory.join("Agent HUD.exe");
    let copy = temp.path().join("copy.exe");
    fs::copy(&executable, &copy).unwrap();
    assert!(check(&directory, &copy).is_none());
    assert!(check(&temp.path().join("old-installation"), &executable).is_none());
    fs::remove_file(directory.join("agent-hud-mcp.exe")).unwrap();
    assert!(check(&directory, &executable).is_none());
}

#[test]
fn registered_target_build_and_malformed_paths_are_refused() {
    let temp = tempfile::tempdir().unwrap();
    let directory = temp.path().join("TaRgEt/release");
    install(&directory);
    assert!(check(&directory, &directory.join("Agent HUD.exe")).is_none());
    for value in [
        "relative",
        "\"unterminated",
        "C:\\app\ninvalid",
        "\"C:\\app\" --extra",
    ] {
        assert!(registered_path(value).is_none());
    }
}

#[test]
fn approved_owner_can_plan_and_install_hooks_only_in_an_isolated_home() {
    use agent_hud_runtime::hooks::{apply_plan, install_plans, InstallContext};
    let temp = tempfile::tempdir().unwrap();
    let directory = temp.path().join("installed");
    install(&directory);
    let executable = directory.join("Agent HUD.exe");
    let home = temp.path().join("home");
    fs::create_dir_all(home.join(".codex")).unwrap();
    let configuration = home.join(".codex/hooks.json");
    fs::write(&configuration, r#"{"unrelated":true}"#).unwrap();
    let context = InstallContext {
        home,
        environment: Default::default(),
        executable: executable.clone(),
        canonical_executable: check(&directory, &executable),
    };
    let plans = install_plans(&context, "codex", "--permission-hook", true);
    assert!(!plans.is_empty());
    for plan in plans {
        apply_plan(&plan.unwrap()).unwrap();
    }
    let installed = fs::read_to_string(&configuration).unwrap();
    assert!(installed.contains("--permission-hook"));
    assert!(installed.contains("Agent HUD.exe"));
    assert!(installed.contains("unrelated"));
}
