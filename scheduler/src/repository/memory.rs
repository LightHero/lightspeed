//! In-memory [`ScheduleRepository`] implementation for tests and single-process use.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use parking_lot::Mutex;

use crate::error::SchedulerError;
use crate::repository::{ScheduleRepository, ScheduleRow};

type Key = (String, String);

#[derive(Debug, Clone)]
struct StoredSchedule {
    next_run_at: SystemTime,
    last_run_at: Option<SystemTime>,
    claimed: bool,
    schedule_fingerprint: String,
}

/// In-memory [`ScheduleRepository`].
#[derive(Debug, Clone, Default)]
pub struct MemoryScheduleRepository {
    inner: Arc<Mutex<HashMap<Key, StoredSchedule>>>,
}

impl MemoryScheduleRepository {
    /// The only constructor. There is no schema to migrate, so this is
    /// effectively `Self::default()` — kept as a named method so the
    /// "construct + ready to use" contract matches
    /// [`PgScheduleRepository::init`](crate::PgScheduleRepository::init).
    pub fn init() -> Self {
        Self::default()
    }
}

/// Transaction handle for [`MemoryScheduleRepository`].
pub struct MemoryTx {
    claimed: Vec<Key>,
    inner: Arc<Mutex<HashMap<Key, StoredSchedule>>>,
}

impl MemoryTx {
    async fn release_claims(&mut self) {
        if self.claimed.is_empty() {
            return;
        }
        let mut guard = self.inner.lock();
        for key in self.claimed.drain(..) {
            if let Some(row) = guard.get_mut(&key) {
                row.claimed = false;
            }
        }
    }
}

impl ScheduleRepository for MemoryScheduleRepository {
    type Tx = MemoryTx;

    async fn begin(&self) -> Result<Self::Tx, SchedulerError> {
        Ok(MemoryTx { claimed: Vec::new(), inner: Arc::clone(&self.inner) })
    }

    async fn commit(&self, mut tx: Self::Tx) -> Result<(), SchedulerError> {
        // Release claims so the rows are pickable again on the next tick.
        tx.release_claims().await;
        Ok(())
    }

    async fn rollback(&self, mut tx: Self::Tx) -> Result<(), SchedulerError> {
        tx.release_claims().await;
        Ok(())
    }

    async fn register(
        &self,
        group: &str,
        name: &str,
        next_run_at: SystemTime,
        schedule_fingerprint: &str,
    ) -> Result<(), SchedulerError> {
        let mut guard = self.inner.lock();
        match guard.get_mut(&(group.to_string(), name.to_string())) {
            Some(stored) => {
                if stored.schedule_fingerprint != schedule_fingerprint {
                    log::warn!(
                        target: "lightspeed_scheduler",
                        "schedule definition changed; re-anchoring next_run_at. \
                         group: {group}, name: {name}, \
                         old_fingerprint: {old:?}, new_fingerprint: {new:?}",
                        old = stored.schedule_fingerprint,
                        new = schedule_fingerprint,
                    );
                    stored.next_run_at = next_run_at;
                    stored.schedule_fingerprint = schedule_fingerprint.to_string();
                }
            }
            None => {
                guard.insert(
                    (group.to_string(), name.to_string()),
                    StoredSchedule {
                        next_run_at,
                        last_run_at: None,
                        claimed: false,
                        schedule_fingerprint: schedule_fingerprint.to_string(),
                    },
                );
            }
        }
        Ok(())
    }

    async fn try_claim_due(
        &self,
        tx: &mut Self::Tx,
        group: &str,
        name: &str,
    ) -> Result<Option<ScheduleRow>, SchedulerError> {
        let now = SystemTime::now();
        let key = (group.to_string(), name.to_string());
        let mut guard = self.inner.lock();
        let row = match guard.get_mut(&key) {
            Some(r) => r,
            None => return Ok(None),
        };
        if row.claimed || row.next_run_at > now {
            return Ok(None);
        }
        row.claimed = true;
        let claimed = ScheduleRow {
            group: group.to_string(),
            name: name.to_string(),
            next_run_at: row.next_run_at,
            last_run_at: row.last_run_at,
        };
        tx.claimed.push(key);
        Ok(Some(claimed))
    }

    async fn advance(
        &self,
        _tx: &mut Self::Tx,
        group: &str,
        name: &str,
        next_run_at: SystemTime,
        last_run_at: SystemTime,
    ) -> Result<(), SchedulerError> {
        let key = (group.to_string(), name.to_string());
        let mut guard = self.inner.lock();
        if let Some(row) = guard.get_mut(&key) {
            row.next_run_at = next_run_at;
            row.last_run_at = Some(last_run_at);
        }
        Ok(())
    }

    async fn time_until_next_due(&self, keys: &[(&str, &str)]) -> Result<Option<Duration>, SchedulerError> {
        let guard = self.inner.lock();
        let now = SystemTime::now();
        let mut min: Option<SystemTime> = None;
        for (group, name) in keys {
            let lookup = (group.to_string(), name.to_string());
            if let Some(row) = guard.get(&lookup) {
                min = Some(match min {
                    Some(m) => m.min(row.next_run_at),
                    None => row.next_run_at,
                });
            }
        }
        Ok(min.map(|t| t.duration_since(now).unwrap_or_default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A timestamp far enough in the past that it's reliably "due" by the
    /// time the test reaches its `try_claim_due` call.
    fn due_in_past() -> SystemTime {
        SystemTime::now() - Duration::from_secs(1)
    }

    const GRP: &str = "g";

    #[tokio::test]
    async fn register_is_idempotent() {
        let repo = MemoryScheduleRepository::init();
        let first = due_in_past();
        repo.register(GRP, "a", first, "").await.unwrap();
        // Second register with a much later time must leave the row alone.
        repo.register(GRP, "a", SystemTime::now() + Duration::from_secs(3600), "").await.unwrap();

        let mut tx = repo.begin().await.unwrap();
        let row = repo
            .try_claim_due(&mut tx, GRP, "a")
            .await
            .unwrap()
            .expect("first register's past time must remain — row must be due");
        assert_eq!(row.group, GRP);
        assert_eq!(row.name, "a");
        assert_eq!(row.next_run_at, first);
        repo.commit(tx).await.unwrap();
    }

    #[tokio::test]
    async fn same_name_in_different_groups_are_isolated() {
        let repo = MemoryScheduleRepository::init();
        repo.register("app-1", "cleanup", due_in_past(), "").await.unwrap();
        repo.register("app-2", "cleanup", SystemTime::now() + Duration::from_secs(60), "").await.unwrap();

        let mut tx = repo.begin().await.unwrap();
        assert!(
            repo.try_claim_due(&mut tx, "app-1", "cleanup").await.unwrap().is_some(),
            "app-1's `cleanup` is due in the past",
        );
        assert!(
            repo.try_claim_due(&mut tx, "app-2", "cleanup").await.unwrap().is_none(),
            "app-2's `cleanup` must NOT have been collapsed onto app-1's row",
        );
        repo.commit(tx).await.unwrap();
    }

    #[tokio::test]
    async fn try_claim_due_returns_none_when_not_due() {
        let repo = MemoryScheduleRepository::init();
        repo.register(GRP, "a", SystemTime::now() + Duration::from_secs(60), "").await.unwrap();

        let mut tx = repo.begin().await.unwrap();
        let row = repo.try_claim_due(&mut tx, GRP, "a").await.unwrap();
        assert!(row.is_none());
        repo.commit(tx).await.unwrap();
    }

    #[tokio::test]
    async fn concurrent_claims_are_exclusive() {
        let repo = MemoryScheduleRepository::init();
        repo.register(GRP, "a", due_in_past(), "").await.unwrap();

        let mut tx1 = repo.begin().await.unwrap();
        let mut tx2 = repo.begin().await.unwrap();

        let claim1 = repo.try_claim_due(&mut tx1, GRP, "a").await.unwrap();
        let claim2 = repo.try_claim_due(&mut tx2, GRP, "a").await.unwrap();

        assert!(claim1.is_some(), "first claim should succeed");
        assert!(claim2.is_none(), "second concurrent claim must be empty");

        repo.commit(tx1).await.unwrap();
        // After the lock is released, a fresh transaction can pick it up.
        let mut tx3 = repo.begin().await.unwrap();
        let claim3 = repo.try_claim_due(&mut tx3, GRP, "a").await.unwrap();
        assert!(claim3.is_some());
        repo.commit(tx3).await.unwrap();
        repo.commit(tx2).await.unwrap();
    }

    #[tokio::test]
    async fn time_until_next_due_returns_smallest_positive_or_zero() {
        let repo = MemoryScheduleRepository::init();
        let now = SystemTime::now();
        repo.register(GRP, "a", now + Duration::from_secs(60), "").await.unwrap();
        repo.register(GRP, "b", now + Duration::from_secs(10), "").await.unwrap();
        // A row in a different group must be ignored when not in the query.
        repo.register("other", "z", due_in_past(), "").await.unwrap();

        let d = repo.time_until_next_due(&[(GRP, "a"), (GRP, "b")]).await.unwrap().expect("at least one matching row");
        assert!(d <= Duration::from_secs(10));
        assert!(d > Duration::from_secs(5));

        // Already-due rows clamp to ZERO rather than going negative.
        repo.register(GRP, "past", due_in_past(), "").await.unwrap();
        let d = repo.time_until_next_due(&[(GRP, "past")]).await.unwrap().expect("`past` must be found");
        assert_eq!(d, Duration::ZERO);

        // Unknown pair returns None.
        assert!(repo.time_until_next_due(&[(GRP, "missing")]).await.unwrap().is_none(),);
        // Wrong group also returns None.
        assert!(repo.time_until_next_due(&[("wrong-group", "a")]).await.unwrap().is_none(),);
        // Empty input returns None.
        assert!(repo.time_until_next_due(&[]).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn advance_updates_next_and_last_run() {
        let repo = MemoryScheduleRepository::init();
        repo.register(GRP, "a", due_in_past(), "").await.unwrap();

        let mut tx = repo.begin().await.unwrap();
        let _ = repo.try_claim_due(&mut tx, GRP, "a").await.unwrap().unwrap();
        // Advance to a future time so the row is no longer due.
        let future = SystemTime::now() + Duration::from_secs(60);
        repo.advance(&mut tx, GRP, "a", future, SystemTime::now()).await.unwrap();
        repo.commit(tx).await.unwrap();

        // Not yet due.
        let mut tx2 = repo.begin().await.unwrap();
        assert!(repo.try_claim_due(&mut tx2, GRP, "a").await.unwrap().is_none(),);
        // Advance back into the past — now claimable again.
        repo.advance(&mut tx2, GRP, "a", due_in_past(), SystemTime::now()).await.unwrap();
        let row = repo.try_claim_due(&mut tx2, GRP, "a").await.unwrap().expect("after backdating, row must be due");
        assert!(row.last_run_at.is_some());
        repo.commit(tx2).await.unwrap();
    }

    /// Re-registering with the same fingerprint must NOT touch `next_run_at`
    /// — that's the long-standing idempotent-register contract.
    #[tokio::test]
    async fn register_with_same_fingerprint_leaves_row_alone() {
        let repo = MemoryScheduleRepository::init();
        let first = due_in_past();
        repo.register(GRP, "a", first, "cron:* * * * * *").await.unwrap();
        // Second call with same fingerprint but a future next_run_at: the
        // existing past `next_run_at` must remain (row stays due).
        repo.register(GRP, "a", SystemTime::now() + Duration::from_secs(3600), "cron:* * * * * *").await.unwrap();

        let mut tx = repo.begin().await.unwrap();
        let row = repo
            .try_claim_due(&mut tx, GRP, "a")
            .await
            .unwrap()
            .expect("first register's past time must remain — row must be due");
        assert_eq!(row.next_run_at, first);
        repo.commit(tx).await.unwrap();
    }

    /// Re-registering with a different fingerprint must replace
    /// `next_run_at` with the supplied value — the change-detection path.
    #[tokio::test]
    async fn register_with_different_fingerprint_reanchors_next_run_at() {
        let repo = MemoryScheduleRepository::init();
        let original_next = due_in_past();
        repo.register(GRP, "a", original_next, "interval:3600.000000000:false").await.unwrap();

        let new_next = SystemTime::now() + Duration::from_secs(60);
        repo.register(GRP, "a", new_next, "interval:60.000000000:false").await.unwrap();

        // After the schedule change, the row is no longer due now.
        let mut tx = repo.begin().await.unwrap();
        assert!(
            repo.try_claim_due(&mut tx, GRP, "a").await.unwrap().is_none(),
            "next_run_at should have been re-anchored to ~now+60s",
        );
        // Backdating the row brings the new fingerprint's row back into view —
        // proves the row's identity wasn't lost during re-anchoring.
        repo.advance(&mut tx, GRP, "a", due_in_past(), SystemTime::now()).await.unwrap();
        assert!(repo.try_claim_due(&mut tx, GRP, "a").await.unwrap().is_some());
        repo.commit(tx).await.unwrap();
    }
}
