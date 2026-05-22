//! Concurrency / contention tests for the SQL-backed [`ScheduleRepository`]
//! implementations.
//!
//! These tests target the row-locking contract that the scheduler depends on:
//! `try_claim_due` must return promptly with `None` when a conflicting tx
//! already holds the row, and must not block on (let alone fail with a
//! lock-wait timeout for) unrelated rows. MySQL is the trickiest backend
//! here — without a usable index on `(group_name, name)` the InnoDB engine
//! degrades to range/table locking and the `FOR UPDATE SKIP LOCKED` clause
//! stops being row-scoped, which manifests as either spurious blocking on
//! independent rows or outright lock-wait timeouts under load. The same
//! tests catch the analogous regression on Postgres.
//!
//! SQLite is excluded because the shared test fixture pins it to a
//! single-connection pool — two concurrent `repo.begin()` calls would
//! deadlock waiting for the pool, not because of anything the scheduler
//! does.
//!
//! Worker counts are bounded by the test pool size (sqlx default: 10
//! connections) so the spawned workers can all hold a connection
//! simultaneously without queueing on the pool. The fixture is shared
//! across tests via `data(false)` (parallel access) — consistent with the
//! rest of this binary's tests.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant, SystemTime};

use lightspeed_scheduler::ScheduleRepository;
use lightspeed_test_utils::tokio_test;
use tokio::sync::Barrier;

use crate::utils::unique_name;
use crate::*;

fn due_in_past() -> SystemTime {
    SystemTime::now() - Duration::from_secs(1)
}

/// N tasks race to claim the same due row. The lock contract says exactly
/// one wins this round; the others must observe `None` immediately (via
/// SKIP LOCKED), not block. A failure here looks like either:
///   * `claims != 1` — SKIP LOCKED isn't actually skipping, so multiple
///     tasks claim the same row (would let two workers fire one job).
///   * `try_claim_due` returns a sqlx lock-wait timeout error — the lock
///     is taking effect, but as a blocking wait rather than a skip.
///
/// The barrier aligns every task at the `try_claim_due` call so the lock
/// contention is real (without it, the first task could commit before the
/// rest even start). N is sized to fit comfortably inside the test pool
/// (sqlx default `max_connections = 10`) — all workers must be able to
/// hold a connection simultaneously, otherwise they'd queue on the pool
/// instead of contending on the row.
#[test]
#[cfg_attr(feature = "sqlite", ignore = "sqlite: single-connection pool serialises naturally")]
fn many_concurrent_claimers_on_one_due_row() {
    tokio_test(async {
        let d = data(false).await;
        let repo = d.0.clone();
        let group = unique_name();
        let name = unique_name();
        repo.register(&group, &name, due_in_past(), "").await.unwrap();

        const N: usize = 8;
        let barrier = Arc::new(Barrier::new(N));
        let claims = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::with_capacity(N);
        for _ in 0..N {
            let repo = repo.clone();
            let group = group.clone();
            let name = name.clone();
            let claims = Arc::clone(&claims);
            let barrier = Arc::clone(&barrier);
            handles.push(tokio::spawn(async move {
                let mut tx = repo.begin().await.expect("begin");
                barrier.wait().await;
                let claimed = repo
                    .try_claim_due(&mut tx, &group, &name)
                    .await
                    .expect("try_claim_due must not error under contention")
                    .is_some();
                if claimed {
                    claims.fetch_add(1, Ordering::SeqCst);
                }
                // Hold the lock long enough that every other task has time to
                // reach `try_claim_due` and observe it locked, not just
                // miss it timing-wise.
                tokio::time::sleep(Duration::from_millis(150)).await;
                repo.commit(tx).await.expect("commit");
            }));
        }
        for h in handles {
            h.await.unwrap();
        }

        assert_eq!(claims.load(Ordering::SeqCst), 1, "exactly one of {N} concurrent claimers should have won the row",);
    });
}

/// N > M workers race against M due rows. The claim+advance flow is the
/// scheduler's actual fire path, so this test mirrors what production
/// load looks like: many ticks, many rows, all in the same group (so
/// they share the leading index column). After all workers settle,
/// exactly M claims must have succeeded — one per row. A failure here is
/// the smoking gun for an unindexed `WHERE group_name = ? AND name = ?`
/// plan, where one worker's lock on row X spreads (via gap locks or a
/// full scan) to rows the other workers would otherwise have been free
/// to claim.
#[test]
#[cfg_attr(feature = "sqlite", ignore = "sqlite: single-connection pool serialises naturally")]
fn many_workers_many_due_rows_each_claimed_once() {
    tokio_test(async {
        let d = data(false).await;
        let repo = d.0.clone();
        let group = unique_name();

        const M: usize = 6;
        const WORKERS: usize = 16;
        let names: Vec<String> = (0..M).map(|_| unique_name()).collect();
        for name in &names {
            repo.register(&group, name, due_in_past(), "").await.unwrap();
        }

        let claims = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::with_capacity(WORKERS);
        for worker in 0..WORKERS {
            let repo = repo.clone();
            let group = group.clone();
            let names = names.clone();
            let claims = Arc::clone(&claims);
            handles.push(tokio::spawn(async move {
                // Each worker walks all M rows, starting at a different
                // offset so workers don't all hammer row 0 first. After a
                // successful claim we advance to a far-future time so the
                // row is no longer due — every claim therefore corresponds
                // to one logical "fire" of that row.
                for k in 0..M {
                    let name = &names[(worker + k) % M];
                    let mut tx = repo.begin().await.expect("begin");
                    let row = repo
                        .try_claim_due(&mut tx, &group, name)
                        .await
                        .expect("try_claim_due must not error under contention");
                    if row.is_some() {
                        repo.advance(
                            &mut tx,
                            &group,
                            name,
                            SystemTime::now() + Duration::from_secs(3600),
                            SystemTime::now(),
                        )
                        .await
                        .expect("advance must not error after a successful claim");
                        claims.fetch_add(1, Ordering::SeqCst);
                    }
                    repo.commit(tx).await.expect("commit");
                }
            }));
        }
        for h in handles {
            h.await.unwrap();
        }

        assert_eq!(
            claims.load(Ordering::SeqCst),
            M,
            "every due row must be claimed exactly once across {WORKERS} workers",
        );
    });
}

/// Two due rows in the **same group**, two workers, one row each. The
/// workers hold their claim for `HOLD` ms before committing. Row-level
/// locking lets them run in parallel — total wall time ≈ `HOLD`. Anything
/// approaching `2 * HOLD` means worker B blocked on worker A's lock,
/// i.e. the lock is acting at a coarser granularity than per-row. That's
/// the MySQL "full table lock" regression to guard against: when the
/// optimizer can't use the `(group_name, name)` functional index, the
/// claim ends up locking the index range/table and unrelated rows in the
/// same group start to serialise.
#[test]
#[cfg_attr(feature = "sqlite", ignore = "sqlite: single-connection pool serialises naturally")]
fn concurrent_claims_on_different_rows_run_in_parallel() {
    tokio_test(async {
        let d = data(false).await;
        let repo = d.0.clone();
        let group = unique_name();
        let name_a = unique_name();
        let name_b = unique_name();
        repo.register(&group, &name_a, due_in_past(), "").await.unwrap();
        repo.register(&group, &name_b, due_in_past(), "").await.unwrap();

        const HOLD: Duration = Duration::from_millis(400);
        let barrier = Arc::new(Barrier::new(2));

        async fn claim_and_hold<R: ScheduleRepository>(repo: R, group: String, name: String, barrier: Arc<Barrier>) {
            let mut tx = repo.begin().await.expect("begin");
            // Align both workers so the second `try_claim_due` happens
            // strictly while the first one's lock is held.
            barrier.wait().await;
            let row = repo.try_claim_due(&mut tx, &group, &name).await.expect("try_claim_due must not error");
            assert!(row.is_some(), "row {name} should be due and claimable");
            tokio::time::sleep(HOLD).await;
            repo.commit(tx).await.expect("commit");
        }

        let start = Instant::now();
        let (ra, rb) = tokio::join!(
            tokio::spawn(claim_and_hold(repo.clone(), group.clone(), name_a.clone(), Arc::clone(&barrier),)),
            tokio::spawn(claim_and_hold(repo.clone(), group.clone(), name_b.clone(), barrier)),
        );
        ra.unwrap();
        rb.unwrap();
        let elapsed = start.elapsed();

        // Parallel: ~HOLD + scheduling jitter. Serial would be ~2*HOLD.
        // The 1.5*HOLD bound catches the "B waited for A" failure mode
        // while tolerating reasonable container/CI jitter.
        assert!(
            elapsed < HOLD + HOLD / 2,
            "claims on two different rows must run in parallel (row-level locking); \
             elapsed {elapsed:?} suggests B blocked on A's lock",
        );
    });
}

/// Worker A claims row X and sleeps for a long while before committing.
/// Worker B *repeatedly* tries to claim row Y (a different name in the
/// same group). Each B attempt must return either `Some` (Y was due) or
/// `None` (Y was advanced) — never an error, and the loop must complete
/// inside A's hold window. Catches the case where a held lock on X
/// starves out claims on Y without quite escalating to a hard timeout.
#[test]
#[cfg_attr(feature = "sqlite", ignore = "sqlite: single-connection pool serialises naturally")]
fn held_claim_does_not_starve_independent_row() {
    tokio_test(async {
        let d = data(false).await;
        let repo = d.0.clone();
        let group = unique_name();
        let slow = unique_name();
        let fast = unique_name();
        repo.register(&group, &slow, due_in_past(), "").await.unwrap();
        repo.register(&group, &fast, due_in_past(), "").await.unwrap();

        const HOLD: Duration = Duration::from_millis(800);

        let repo_a = repo.clone();
        let group_a = group.clone();
        let slow_a = slow.clone();
        let a = tokio::spawn(async move {
            let mut tx = repo_a.begin().await.expect("begin");
            assert!(repo_a.try_claim_due(&mut tx, &group_a, &slow_a).await.expect("claim slow").is_some(),);
            tokio::time::sleep(HOLD).await;
            repo_a.commit(tx).await.expect("commit slow");
        });

        // Give A a moment to grab its lock before B starts probing.
        tokio::time::sleep(Duration::from_millis(50)).await;

        let start = Instant::now();
        let mut attempts = 0usize;
        let mut fast_claimed = false;
        while start.elapsed() < HOLD - Duration::from_millis(100) {
            let mut tx = repo.begin().await.expect("begin fast");
            let row = repo
                .try_claim_due(&mut tx, &group, &fast)
                .await
                .expect("try_claim_due on independent row must not error while another row is locked");
            if row.is_some() {
                fast_claimed = true;
                repo.advance(&mut tx, &group, &fast, SystemTime::now() + Duration::from_secs(3600), SystemTime::now())
                    .await
                    .expect("advance fast");
            }
            repo.commit(tx).await.expect("commit fast");
            attempts += 1;
            tokio::time::sleep(Duration::from_millis(20)).await;
        }

        a.await.unwrap();

        assert!(
            fast_claimed,
            "row `fast` should have been claimable while `slow` was locked (took {attempts} attempts)",
        );
    });
}
