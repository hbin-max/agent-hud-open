use super::*;

fn parse_servers(processes: Vec<crate::os::ProcessInfo>) -> Vec<Server> {
    super::parse_servers(processes, &HashMap::new())
}

#[test]
fn saved_token_restores_hub_discovery_and_keeps_command_line_precedence() {
    let root = tempfile::tempdir().unwrap();
    let daemon = root.path().join(".gemini/antigravity/daemon");
    std::fs::create_dir_all(&daemon).unwrap();
    for (file, body) in [
        ("hub.json", r#"{"pid":1,"csrfToken":"saved-1"}"#),
        ("other.json", r#"{"pid":2,"csrfToken":"saved-2"}"#),
        ("stale.json", r#"{"pid":9,"csrfToken":"saved-9"}"#),
        ("bad.json", "{"),
        ("control.json", r#"{"pid":3,"csrfToken":"bad\nvalue"}"#),
    ] {
        std::fs::write(daemon.join(file), body).unwrap();
    }
    std::fs::write(
        daemon.join("oversized.json"),
        format!(r#"{{"pid":4,"csrfToken":"{}"}}"#, "x".repeat(5000)),
    )
    .unwrap();
    let saved = saved_tokens(root.path(), &crate::os::Environment::new());
    assert_eq!(saved.len(), 3);
    let exe = r"C:\Antigravity\language_server.exe";
    let found = super::parse_servers(vec![
        process(1, exe, "server --standalone --app_data_dir antigravity --read_host_bridge_token_from_stdin"),
        process(2, exe, "server --csrf_token command-2"),
        process(3, exe, "server --app_data_dir antigravity"),
        process(9, r"C:\Other\unrelated.exe", "unrelated"),
    ], &saved);
    assert_eq!(
        found
            .iter()
            .map(|s| (s.pid, s.token.as_str()))
            .collect::<Vec<_>>(),
        [(1, "saved-1"), (2, "command-2")]
    );
}

fn process(pid: u32, executable: &str, command: &str) -> crate::os::ProcessInfo {
    crate::os::ProcessInfo {
        pid,
        executable: executable.into(),
        command: command.into(),
    }
}

#[test]
fn discovers_all_authenticated_native_and_cli_servers() {
    let found = parse_servers(vec![
        process(
            1,
            "/Applications/Antigravity.app/language_server",
            "server --csrf_token fixture-1",
        ),
        process(
            2,
            "/u/.gemini/antigravity-cli/language_server",
            "server --csrf_token fixture-2",
        ),
        process(3, "/bin/agy", "agy"),
        process(
            4,
            "/unrelated/language_server",
            "server --csrf_token fixture-4",
        ),
        process(
            5,
            "/Applications/Antigravity IDE.app/language-server",
            "server --csrf_token fixture-5",
        ),
    ]);
    assert_eq!(
        found.iter().map(|server| server.pid).collect::<Vec<_>>(),
        [1, 2, 5]
    );
}

#[test]
fn discovers_windows_language_server_and_cli_with_native_paths() {
    let found = parse_servers(vec![
        process(
            1,
            r"C:\Program Files\Antigravity\language_server.exe",
            "server --app_data_dir antigravity --csrf_token fixture",
        ),
        process(2, r"C:\Tools\agy.exe", "agy.exe --csrf_token fixture"),
        process(
            3,
            r"C:\Other\language_server.exe",
            "server --app_data_dir unrelated --csrf_token fixture",
        ),
    ]);
    assert_eq!(
        found.iter().map(|server| server.pid).collect::<Vec<_>>(),
        [1, 2]
    );
}

#[test]
fn tokenless_cli_fallback_uses_the_executable_not_command_arguments() {
    let root = tempfile::tempdir().unwrap();
    let executable = root.path().join("installed/agy");
    let executable = executable.to_str().unwrap();
    let found = cli_from(vec![
        process(1, "/bin/sh", "sh /bin/agy"),
        process(2, executable, "agy"),
    ]);
    assert_eq!(found.as_deref(), Some(executable));
    assert!(cli_from(vec![process(1, "agy", "agy")]).is_none());
}
