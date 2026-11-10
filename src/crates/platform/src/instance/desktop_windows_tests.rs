use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Barrier,
};

fn isolated() -> Identity {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    identity(&format!(
        "test-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ))
}

#[test]
fn mutex_is_exclusive_and_released_on_drop() {
    let id = isolated();
    let owner = acquire(&id.mutex).unwrap().unwrap();
    assert!(acquire(&id.mutex).unwrap().is_none());
    drop(owner);
    assert!(acquire(&id.mutex).unwrap().is_some());
}

#[test]
fn concurrent_launches_elect_one_owner() {
    let id = isolated();
    let gate = Arc::new(Barrier::new(8));
    let workers: Vec<_> = (0..8)
        .map(|_| {
            let name = id.mutex.clone();
            let gate = Arc::clone(&gate);
            std::thread::spawn(move || {
                gate.wait();
                let owner = acquire(&name).unwrap();
                // Every launch must finish acquisition before the winner drops.
                gate.wait();
                owner.is_some()
            })
        })
        .collect();
    assert_eq!(
        workers
            .into_iter()
            .map(|worker| usize::from(worker.join().unwrap()))
            .sum::<usize>(),
        1
    );
}

#[test]
fn duplicate_wakes_owner_and_does_not_acquire() {
    let id = isolated();
    let duplicate = Identity {
        mutex: id.mutex.clone(),
        endpoint: id.endpoint.clone(),
    };
    let owner = claim_identity(id, Duration::ZERO).unwrap().unwrap();
    let shown = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&shown);
    let _running = owner
        .listen(Arc::new(move || {
            count.fetch_add(1, Ordering::SeqCst);
        }))
        .unwrap();
    assert!(claim_identity(duplicate, Duration::from_secs(2))
        .unwrap()
        .is_none());
    assert_eq!(shown.load(Ordering::SeqCst), 1);
}

#[test]
fn duplicate_waits_for_owner_to_finish_startup() {
    let id = isolated();
    let duplicate = Identity {
        mutex: id.mutex.clone(),
        endpoint: id.endpoint.clone(),
    };
    let owner = claim_identity(id, Duration::ZERO).unwrap().unwrap();
    let worker = std::thread::spawn(move || {
        claim_identity(duplicate, Duration::from_secs(3))
            .unwrap()
            .is_none()
    });
    std::thread::sleep(Duration::from_millis(150));
    let _running = owner.listen(Arc::new(|| {})).unwrap();
    assert!(worker.join().unwrap());
}

#[test]
fn unresponsive_owner_fails_closed_with_bounded_wait() {
    let id = isolated();
    let _owner = acquire(&id.mutex).unwrap().unwrap();
    let start = Instant::now();
    assert!(claim_identity(id, Duration::from_millis(100)).is_err());
    assert!(start.elapsed() < Duration::from_secs(2));
}

#[test]
fn different_users_have_independent_identities() {
    let a = identity("S-1-5-21-a");
    let b = identity("S-1-5-21-b");
    assert_ne!(a.mutex, b.mutex);
    assert_ne!(a.endpoint, b.endpoint);
}

// Invoked as a separate process by cross_process_and_copy_use_the_same_guard.
#[test]
fn child_probe() {
    let Ok(name) = std::env::var("AGENTHUD_TEST_MUTEX") else {
        return;
    };
    let endpoint = std::env::var("AGENTHUD_TEST_ENDPOINT").unwrap();
    assert!(
        claim_identity(
            Identity {
                mutex: name,
                endpoint
            },
            Duration::from_secs(3)
        )
        .unwrap()
        .is_none(),
        "another executable/data directory must not bypass the owner"
    );
}

#[test]
fn cross_process_and_copy_use_the_same_guard() {
    let id = isolated();
    let owner = claim_identity(
        Identity {
            mutex: id.mutex.clone(),
            endpoint: id.endpoint.clone(),
        },
        Duration::ZERO,
    )
    .unwrap()
    .unwrap();
    let shown = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&shown);
    let _running = owner
        .listen(Arc::new(move || {
            count.fetch_add(1, Ordering::SeqCst);
        }))
        .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let copied = temp.path().join("Agent HUD test copy.exe");
    std::fs::copy(std::env::current_exe().unwrap(), &copied).unwrap();
    let result = std::process::Command::new(&copied)
        .args([
            "--exact",
            "instance::desktop::tests::child_probe",
            "--nocapture",
        ])
        .env("AGENTHUD_TEST_MUTEX", &id.mutex)
        .env("AGENTHUD_TEST_ENDPOINT", &id.endpoint)
        .env("LOCALAPPDATA", temp.path().join("redirected"))
        .env("AGENTHUD_DATA_DIR", temp.path().join("other-data"))
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
    assert!(
        String::from_utf8_lossy(&result.stdout).contains("1 passed"),
        "the child probe must actually run"
    );
    assert_eq!(
        shown.load(Ordering::SeqCst),
        1,
        "the copied process must wake the original owner"
    );
}

#[test]
fn child_owner() {
    use std::io::Write;
    let Ok(name) = std::env::var("AGENTHUD_TEST_OWNER") else {
        return;
    };
    let _owner = acquire(&name).unwrap().unwrap();
    println!("OWNER_READY");
    std::io::stdout().flush().unwrap();
    std::thread::sleep(Duration::from_secs(30));
}

#[test]
fn crashing_owner_releases_guard_without_deleting_files() {
    use std::io::{BufRead, BufReader};
    use std::process::{Child, Command, Stdio};
    struct ChildOwner(Child);
    impl Drop for ChildOwner {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let id = isolated();
    let mut child = ChildOwner(
        Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "instance::desktop::tests::child_owner",
                "--nocapture",
            ])
            .env("AGENTHUD_TEST_OWNER", &id.mutex)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let reader = BufReader::new(child.0.stdout.take().unwrap());
    let ready = reader
        .lines()
        .any(|line| line.unwrap().contains("OWNER_READY"));
    assert!(
        ready,
        "child must hold the actual kernel object before testing its death"
    );
    assert!(acquire(&id.mutex).unwrap().is_none());
    drop(child);
    assert!(acquire(&id.mutex).unwrap().is_some());
}
