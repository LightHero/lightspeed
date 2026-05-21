//! Repository-level integration tests against a real Postgres backend.

use std::time::{Duration, SystemTime};

use lightspeed_scheduler::ScheduleRepository;
use lightspeed_test_utils::tokio_test;

use crate::utils::unique_name;
use crate::*;

/// A timestamp far enough in the past that the row is reliably due against
/// the DB's `NOW()` by the time the test reaches its `try_claim_due` call.
fn due_in_past() -> SystemTime {
    SystemTime::now() - Duration::from_secs(1)
}

#[test]
fn repository_can_be_initialised_twice() {
    tokio_test(async {
        // The fixture has already initialised the repository once. Calling
        // `init` again — through `start_repo()` — must succeed against the
        // same backing. For Postgres this exercises sqlx's migration tracker
        // (re-running migrations is a no-op); for the in-memory backend it's
        // just "init() can be called repeatedly".
        let _first = data(false).await;
        let second = start_repo().await;

        // The freshly-initialised handle is usable: a register-then-claim
        // round-trip succeeds.
        let group = unique_name();
        let name = unique_name();
        second.register(&group, &name, due_in_past(), "").await.unwrap();

        let mut tx = second.begin().await.unwrap();
        assert!(
            second
                .try_claim_due(&mut tx, &group, &name)
                .await
                .unwrap()
                .is_some(),
            "second init's handle must be fully usable",
        );
        second.commit(tx).await.unwrap();
    });
}

#[test]
fn register_inserts_row_then_no_ops() {
    tokio_test(async {
        let d = data(false).await;
        let repo = &d.0;
        let group = unique_name();
        let name = unique_name();

        // First register: in the past so the row is due.
        let first = due_in_past();
        repo.register(&group, &name, first, "").await.unwrap();

        // Second register with a far-future time must leave the row unchanged.
        repo.register(&group, &name, SystemTime::now() + Duration::from_secs(3600), "")
            .await
            .unwrap();

        let mut tx = repo.begin().await.unwrap();
        let row = repo
            .try_claim_due(&mut tx, &group, &name)
            .await
            .unwrap()
            .expect("first register's past time must remain — row must be due");
        assert_eq!(row.group, group);
        assert_eq!(row.name, name);
        assert!(
            row.next_run_at < SystemTime::now() + Duration::from_secs(60),
            "first register's next_run_at must win, got {:?}",
            row.next_run_at,
        );
        repo.commit(tx).await.unwrap();
    });
}

#[test]
fn same_name_in_different_groups_are_isolated() {
    tokio_test(async {
        let d = data(false).await;
        let repo = &d.0;
        let group_a = unique_name();
        let group_b = unique_name();
        let name = unique_name();

        repo.register(&group_a, &name, due_in_past(), "").await.unwrap();
        repo.register(&group_b, &name, SystemTime::now() + Duration::from_secs(60), "")
            .await
            .unwrap();

        let mut tx = repo.begin().await.unwrap();
        assert!(
            repo.try_claim_due(&mut tx, &group_a, &name)
                .await
                .unwrap()
                .is_some(),
            "{group_a}/{name} is due in the past",
        );
        assert!(
            repo.try_claim_due(&mut tx, &group_b, &name)
                .await
                .unwrap()
                .is_none(),
            "{group_b}/{name} must NOT have been collapsed onto group_a's row",
        );
        repo.commit(tx).await.unwrap();
    });
}

#[test]
fn try_claim_due_returns_none_when_not_due() {
    tokio_test(async {
        let d = data(false).await;
        let repo = &d.0;
        let group = unique_name();
        let name = unique_name();
        repo.register(&group, &name, SystemTime::now() + Duration::from_secs(60), "")
            .await
            .unwrap();

        let mut tx = repo.begin().await.unwrap();
        let row = repo.try_claim_due(&mut tx, &group, &name).await.unwrap();
        assert!(row.is_none());
        repo.commit(tx).await.unwrap();
    });
}

#[test]
fn try_claim_due_returns_row_when_due() {
    tokio_test(async {
        let d = data(false).await;
        let repo = &d.0;
        let group = unique_name();
        let name = unique_name();
        repo.register(&group, &name, due_in_past(), "").await.unwrap();

        let mut tx = repo.begin().await.unwrap();
        let row = repo
            .try_claim_due(&mut tx, &group, &name)
            .await
            .unwrap()
            .expect("schedule should be due");
        assert_eq!(row.group, group);
        assert_eq!(row.name, name);
        assert!(row.last_run_at.is_none());
        repo.commit(tx).await.unwrap();
    });
}

// SQLite has no `FOR UPDATE SKIP LOCKED`, and the test fixture's pool is
// single-connection by design, so two concurrent `repo.begin()` calls would
// deadlock waiting for the pool. The scheduler's sqlite backend is single-
// process by contract — concurrent claim arbitration is out of scope there.
#[test]
#[cfg_attr(feature = "sqlite", ignore = "sqlite: single-connection pool, no row locks")]
fn for_update_skip_locked_serialises_concurrent_claims() {
    tokio_test(async {
        let d = data(false).await;
        let repo = &d.0;
        let group = unique_name();
        let name = unique_name();
        repo.register(&group, &name, due_in_past(), "").await.unwrap();

        // Open two concurrent transactions. The first one must lock the row;
        // the second one must see it as locked and skip it.
        let mut tx1 = repo.begin().await.unwrap();
        let mut tx2 = repo.begin().await.unwrap();

        let claim1 = repo.try_claim_due(&mut tx1, &group, &name).await.unwrap();
        let claim2 = repo.try_claim_due(&mut tx2, &group, &name).await.unwrap();

        assert!(claim1.is_some(), "first claim should succeed");
        assert!(claim2.is_none(), "concurrent claim must be skipped");

        // After tx1 commits without advancing, tx2 should see the row again
        // (it was unlocked, not advanced).
        repo.commit(tx1).await.unwrap();
        let retry = repo.try_claim_due(&mut tx2, &group, &name).await.unwrap();
        assert!(retry.is_some(), "claim must succeed after lock released");
        repo.commit(tx2).await.unwrap();
    });
}

#[test]
fn advance_updates_next_and_last_run() {
    tokio_test(async {
        let d = data(false).await;
        let repo = &d.0;
        let group = unique_name();
        let name = unique_name();
        repo.register(&group, &name, due_in_past(), "").await.unwrap();

        let mut tx = repo.begin().await.unwrap();
        let _ = repo
            .try_claim_due(&mut tx, &group, &name)
            .await
            .unwrap()
            .expect("schedule should be due");
        // Advance to a future time so the row stops being due.
        let future = SystemTime::now() + Duration::from_secs(60);
        repo.advance(&mut tx, &group, &name, future, SystemTime::now())
            .await
            .unwrap();
        repo.commit(tx).await.unwrap();

        // Not yet due.
        let mut tx2 = repo.begin().await.unwrap();
        assert!(
            repo.try_claim_due(&mut tx2, &group, &name)
                .await
                .unwrap()
                .is_none(),
            "must not be due before advanced next_run_at",
        );
        // Move next_run_at back into the past — now claimable again, and
        // last_run_at must be populated from the previous advance.
        repo.advance(&mut tx2, &group, &name, due_in_past(), SystemTime::now())
            .await
            .unwrap();
        let row = repo
            .try_claim_due(&mut tx2, &group, &name)
            .await
            .unwrap()
            .expect("after backdating, row must be due");
        assert!(row.last_run_at.is_some());
        repo.commit(tx2).await.unwrap();
    });
}

#[test]
fn rollback_releases_lock_without_advance() {
    tokio_test(async {
        let d = data(false).await;
        let repo = &d.0;
        let group = unique_name();
        let name = unique_name();
        repo.register(&group, &name, due_in_past(), "").await.unwrap();

        let mut tx = repo.begin().await.unwrap();
        let claim = repo.try_claim_due(&mut tx, &group, &name).await.unwrap();
        assert!(claim.is_some());
        repo.rollback(tx).await.unwrap();

        // After rollback the row should be claimable again.
        let mut tx2 = repo.begin().await.unwrap();
        let again = repo.try_claim_due(&mut tx2, &group, &name).await.unwrap();
        assert!(again.is_some(), "rollback must release the lock");
        repo.commit(tx2).await.unwrap();
    });
}

#[test]
fn time_until_next_due_uses_db_clock_and_respects_groups() {
    tokio_test(async {
        let d = data(false).await;
        let repo = &d.0;
        let group = unique_name();
        let other = unique_name();

        let a = unique_name();
        let b = unique_name();
        let c = unique_name();

        repo.register(&group, &a, SystemTime::now() + Duration::from_secs(60), "")
            .await
            .unwrap();
        repo.register(&group, &b, SystemTime::now() + Duration::from_secs(10), "")
            .await
            .unwrap();
        repo.register(&group, &c, SystemTime::now() + Duration::from_secs(30), "")
            .await
            .unwrap();
        // A row in another group must be ignored by a query scoped to `group`.
        repo.register(&other, &b, due_in_past(), "").await.unwrap();

        // Smallest of a/b/c is `b` ~10s out.
        let d_until = repo
            .time_until_next_due(&[
                (group.as_str(), a.as_str()),
                (group.as_str(), b.as_str()),
                (group.as_str(), c.as_str()),
            ])
            .await
            .unwrap()
            .expect("at least one row matches");
        assert!(d_until <= Duration::from_secs(10), "got {d_until:?}");
        assert!(d_until > Duration::from_secs(5));

        // Past-due rows clamp to ZERO rather than going negative.
        let overdue = unique_name();
        repo.register(&group, &overdue, due_in_past(), "")
            .await
            .unwrap();
        let d_until = repo
            .time_until_next_due(&[(group.as_str(), overdue.as_str())])
            .await
            .unwrap()
            .expect("`overdue` must be found");
        assert_eq!(d_until, Duration::ZERO);

        // Unknown pair + empty slice both return None.
        assert!(
            repo.time_until_next_due(&[(group.as_str(), "__definitely_missing__")])
                .await
                .unwrap()
                .is_none(),
        );
        assert!(repo.time_until_next_due(&[]).await.unwrap().is_none());

        // A name that exists in another group must NOT match a query scoped
        // to `group`. This is the key guarantee of the compound key.
        assert!(
            repo.time_until_next_due(&[("__missing_group__", b.as_str())])
                .await
                .unwrap()
                .is_none(),
            "wrong group must not see `b`",
        );
    });
}
