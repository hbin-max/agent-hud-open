//! One desktop per Windows user, including copies with redirected AppData.
//! The mutex and wake-up endpoint use the user's SID, never a file path.
//! Collector locks remain responsible for data and hook exclusivity.
use super::OnShow;
use crate::os::pipe::{self, user_security, user_sid};
use serde_json::{json, Value};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE},
    Security::SECURITY_ATTRIBUTES,
    System::Threading::CreateMutexW,
};

const LIMIT: usize = 64;
const STARTUP_WAIT: Duration = Duration::from_secs(10);

struct Identity {
    mutex: String,
    endpoint: String,
}

fn identity(sid: &str) -> Identity {
    Identity {
        mutex: format!("Global\\agenthud-desktop-v1-{sid}"),
        endpoint: format!("\\\\.\\pipe\\agenthud-desktop-v1-{sid}"),
    }
}

// No thread owns this mutex: its kernel object lives while this handle lives.
// Store the handle as an integer so the guard can live in Tauri's managed state.
// Only CloseHandle uses it; no thread-affine wait/release operations occur.
struct Presence(isize);

impl Drop for Presence {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0 as HANDLE) };
    }
}

pub struct Guard {
    presence: Presence,
    endpoint: String,
}

pub struct Running {
    // Drop the listener before the presence handle, so a new owner cannot
    // race the old endpoint during orderly shutdown.
    _endpoint: pipe::Server,
    _presence: Presence,
}

fn acquire(name: &str) -> Result<Option<Presence>, String> {
    let descriptor = user_security(false)?;
    let attributes = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor.0,
        bInheritHandle: 0,
    };
    let wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    let handle = unsafe { CreateMutexW(&attributes, 0, wide.as_ptr()) };
    // Capture immediately; any subsequent Win32 call can overwrite last error.
    let error = unsafe { GetLastError() };
    if handle.is_null() {
        return Err(format!(
            "Desktop instance guard unavailable (Windows error {error})"
        ));
    }
    let presence = Presence(handle as isize);
    if error == ERROR_ALREADY_EXISTS {
        drop(presence);
        Ok(None)
    } else {
        Ok(Some(presence))
    }
}

/// None means the existing desktop acknowledged the show request. A failure
/// never falls back to launching an unprotected second desktop.
pub fn claim() -> Result<Option<Guard>, String> {
    claim_identity(identity(&user_sid()?), STARTUP_WAIT)
}

fn claim_identity(identity: Identity, timeout: Duration) -> Result<Option<Guard>, String> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(presence) = acquire(&identity.mutex)? {
            return Ok(Some(Guard {
                presence,
                endpoint: identity.endpoint,
            }));
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(
                "Agent HUD is already running but did not respond. Quit that copy and try again."
                    .into(),
            );
        }
        if pipe::request_named(
            &identity.endpoint,
            &json!("show"),
            LIMIT,
            LIMIT,
            remaining.min(Duration::from_millis(500)),
            || false,
        )
        .ok()
        .flatten()
            == Some(json!("shown"))
        {
            return Ok(None);
        }
        // The owner may still be preparing its UI, or may have crashed. Retry
        // both acquisition and handover within a bounded startup interval.
        std::thread::sleep(remaining.min(Duration::from_millis(50)));
    }
}

impl Guard {
    pub fn listen(self, on_show: OnShow) -> Result<Running, String> {
        let endpoint = pipe::Server::start_named(
            &self.endpoint,
            LIMIT,
            LIMIT,
            Arc::new(move |request: Value, _| {
                (request == json!("show")).then(|| {
                    on_show();
                    json!("shown")
                })
            }),
        )?;
        Ok(Running {
            _endpoint: endpoint,
            _presence: self.presence,
        })
    }
}

#[cfg(test)]
#[path = "desktop_windows_tests.rs"]
mod tests;
