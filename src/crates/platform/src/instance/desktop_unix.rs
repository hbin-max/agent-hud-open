//! Other hosts keep their existing collector-lock startup policy.
use super::OnShow;

pub struct Guard;
pub struct Running;

pub fn claim() -> Result<Option<Guard>, String> {
    Ok(Some(Guard))
}

impl Guard {
    pub fn listen(self, _on_show: OnShow) -> Result<Running, String> {
        Ok(Running)
    }
}
