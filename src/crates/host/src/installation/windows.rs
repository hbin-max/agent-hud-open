use std::path::{Path, PathBuf};
use windows_registry::CURRENT_USER;

// Matches the current-user NSIS installer; custom installation directories
// are read from its record, never LOCALAPPDATA or a hook payload.
const KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Agent HUD";

pub(crate) fn canonical() -> Option<PathBuf> {
    let key = CURRENT_USER.open(KEY).ok()?;
    if key.get_string("DisplayName").ok()? != "Agent HUD" {
        return None;
    }
    approved(
        &key.get_string("InstallLocation").ok()?,
        &key.get_string("DisplayIcon").ok()?,
        &key.get_string("UninstallString").ok()?,
        &std::env::current_exe().ok()?,
    )
}

fn registered_path(value: &str) -> Option<PathBuf> {
    let value = value.trim();
    let value = if value.starts_with('"') {
        value.strip_prefix('"')?.strip_suffix('"')?
    } else {
        value
    };
    let path = PathBuf::from(value);
    (path.is_absolute() && !value.contains(['\0', '\r', '\n', '"'])).then_some(path)
}

fn transient(path: &Path) -> bool {
    path.components().any(|part| {
        part.as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case("target")
    })
}

fn approved(location: &str, icon: &str, uninstall: &str, current: &Path) -> Option<PathBuf> {
    let directory = registered_path(location)?;
    let executable = directory.join("Agent HUD.exe");
    // Preserve the hook module's exact owner-path contract. Copies, aliases,
    // target builds and stale installer entries remain unavailable.
    if executable != current
        || registered_path(icon)? != executable
        || registered_path(uninstall)? != directory.join("uninstall.exe")
        || transient(&executable)
    {
        return None;
    }
    for name in [
        "Agent HUD.exe",
        "uninstall.exe",
        "agent-hud-engine.exe",
        "agent-hud-mcp.exe",
    ] {
        if !directory.join(name).is_file() {
            return None;
        }
    }
    let resolved = executable.canonicalize().ok()?;
    if transient(&resolved) {
        return None;
    }
    Some(executable)
}

#[cfg(test)]
mod tests;
