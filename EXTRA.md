# EXTRA practice — guidance and reference paths
Attempt each question before opening the solution. These additions extend the existing outcomes, not course numbering.

## E1 — Remainders and numeric meaning (O2, L09/L27)
Add a filter that sums values whose timestamp falls in [from,to). Test zero length, five values, boundary timestamps and invalid column lengths. Hint: filter paired borrowed slices before considering SIMD; preserve equal-length validation. Do not compare arbitrary f64 sums by exact equality unless the numerical contract permits it.
Reference: extras-solutions/src/lib.rs bounded_sum plus its boundary tests. Extend the benchmark only after correctness; retain conversion/filter costs separately.

## E2 — Ready instance and cancellation (O3, L14/L20)
Build a test Service whose clone starts unready, whose poll_ready makes that instance ready and whose call refuses an unready instance. Wrap it with CountLayer and drive one call. Then drop a pending response future and explain started/completed.
Hint: avoid cloning the inner service between readiness and call. Reference: aspects/tests/join_contract.rs. A cancelled call has no observed inner Result.

## E3 — Async macro refusal (O3, L16)
Add an audited async Result function using question-mark on an error. Verify the original error and a normal result. Hint: the generated awaited block must capture early return/question-mark, with no retry or replacement.
Reference: aspects/tests/join_contract.rs. Before extending the macro to other signatures, add compile-fail coverage in a qualified toolchain; unsupported contracts remain refused.

## E4 — Tail gap and tenancy (O4, L21)
Run two authenticated WebSocket clients for different tenants. Ingest one tenant's batch, slow that subscriber and exceed the128-batch retention capacity. Record isolation, resync_required and the following query/reconnect.
Hint: a display stream has a cursor and can miss history. Reference: axum-api stream_tail, filter before serialization, bounded send and Lagged branch. This is an integration observation, not a prefilled test pass. Implement a sequence/replay contract only if your product actually requires it.

## E5 — Corrupt snapshot preservation (O6, L32)
Put malformed bytes under course-ui-v1 in a temporary learner store. Reopen Model and require an error without replacing the bytes. Then recover from a known-good retained backup in a separate copy.
Hint: parse/validate before any write. Reference: desktop-core/tests/corrupt_snapshot.rs. Do not discard invalid history silently.

## E6 — No dedup record for an unapplied effect (O5, L28)
Apply a valid envelope for a missing order and inspect that its processed_event row was rolled back. Create the accepted order, replay and prove exactly one matching row.
Hint: both mutations must share one transaction. Reference: event-svc/tests/missing_order.rs failed_effect_does_not_consume_idempotency_key. Run only in isolated migrated course_local. Extend to acknowledgement-loss in a real JetStream run; a database unit observation alone does not prove broker recovery.