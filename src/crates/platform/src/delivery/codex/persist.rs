//! Preserve the effort resolved by Codex's effective config for a new thread.
use super::*;
use serde_json::{json, Value};

pub(crate) fn persist_started_thread(
    id: &str,
    title: &str,
    started: &Value,
    mut mutate: impl FnMut(&str, Value) -> Result<Value>,
) -> Result<()> {
    mutate("thread/name/set", json!({"threadId": id, "name": title}))?;
    if let Some(effort) = started["reasoningEffort"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if let Err(error) = mutate(
            "thread/settings/update",
            json!({"threadId":id,"effort":effort}),
        ) {
            // Older clients need not implement this method to accept the first message.
            eprintln!("Codex thread/settings/update for {id} skipped: {error}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn materializes_then_persists_effort_and_skips_missing_values() {
        for effort in [json!("ultra"), Value::Null, json!(" ")] {
            let mut calls = Vec::new();
            persist_started_thread(
                "t",
                "Check",
                &json!({"reasoningEffort":effort}),
                |method, params| {
                    calls.push((method.to_owned(), params));
                    Ok(json!({}))
                },
            )
            .unwrap();
            assert_eq!(
                calls[0],
                (
                    "thread/name/set".into(),
                    json!({"threadId":"t","name":"Check"})
                )
            );
            if effort == "ultra" {
                assert_eq!(
                    calls[1],
                    (
                        "thread/settings/update".into(),
                        json!({"threadId":"t","effort":"ultra"})
                    )
                );
            } else {
                assert_eq!(calls.len(), 1);
            }
        }
    }

    #[test]
    fn settings_failures_are_best_effort_but_materialization_is_required() {
        for failure in [
            Failure::Rejected("unsupported".into()),
            Failure::TimedOut,
            Failure::AcceptanceUncertain,
            Failure::Io("pipe".into()),
        ] {
            assert!(persist_started_thread(
                "t",
                "Check",
                &json!({"reasoningEffort":"high"}),
                |method, _| {
                    if method == "thread/settings/update" {
                        Err(failure.clone())
                    } else {
                        Ok(json!({}))
                    }
                }
            )
            .is_ok());
        }
        assert!(
            persist_started_thread("t", "Check", &json!({}), |_, _| Err(Failure::TimedOut))
                .is_err()
        );
    }
}
