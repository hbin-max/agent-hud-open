//! Session heads shared by checked-in clients and out-of-window desktop threads.
use super::*;

pub(crate) fn party_any(snapshot: &Value, extras: &[Value], client: &str, id: &str) -> Party {
    Party {
        client: client.into(),
        session_id: id.into(),
        title: find_in_any(snapshot, extras, client, id)
            .and_then(|session| session["title"].as_str())
            .map(str::to_owned),
    }
}

/// Like `find`, also considering live extra sessions outside the snapshot.
pub(crate) fn find_any<'a>(
    snapshot: &'a Value,
    extras: &'a [Value],
    id: &str,
) -> Option<&'a Value> {
    let mut found = snapshot["sessions"]
        .as_array()
        .into_iter()
        .flatten()
        .chain(extras.iter())
        .filter(|session| session["id"].as_str().is_some_and(|s| same_session(s, id)));
    let first = found.next()?;
    found
        .all(|other| other["source"] == first["source"] && other["id"] == first["id"])
        .then_some(first)
}

/// Like `find_in`, also considering live extra sessions outside the snapshot.
pub(crate) fn find_in_any<'a>(
    snapshot: &'a Value,
    extras: &'a [Value],
    client: &str,
    id: &str,
) -> Option<&'a Value> {
    find_in(snapshot, client, id).or_else(|| {
        extras.iter().find(|session| {
            session["source"] == client
                && session["id"].as_str().is_some_and(|s| same_session(s, id))
        })
    })
}
