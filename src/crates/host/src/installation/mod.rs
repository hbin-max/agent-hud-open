//! The desktop packaging contract. Only a registered installation may own hooks.
#[cfg(windows)]
mod windows;
#[cfg(windows)]
use windows as target;
#[cfg(not(windows))]
mod other;
#[cfg(not(windows))]
use other as target;

pub(crate) use target::canonical;
