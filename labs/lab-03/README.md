# L3 — JetStream consumer with retry, DLQ and idempotency
Outcome O5; lessons25,28,29. Requires L2 migrated learner PostgreSQL, API token, local NATS JetStream and Jaeger.

1. Implement the starter decision policy and preserve refusal at attempt0.
2. Start API and POST /orders with order_id=order-1, units=2. Inspect accepted order and unsent outbox row.
3. Run cargo run -p event-svc -- publish. This finite dispatcher processes at most32 rows. Run it again when new rows exist; it is not a background production scheduler.
4. Run cargo run -p event-svc -- consume. Refresh /orders/order-1 until fulfilled. Inspect one processed_event row.
5. Replay the same accepted outbox payload through an approved local NATS CLI or a declared replay fixture. Broker dedup may suppress a near-term repeat; test durable DB dedup separately with the explicit event-svc PostgreSQL test.
6. Change units under the same event key and prove conflict without another effect. Stop PostgreSQL during an attempt, recover it and observe delayed retry.
7. Send a bounded malformed envelope, inspect durable dead_letter plus orders.dlq and original ack state. Exhaustion/oversized inputs require explicit operator quarantine; inspect retained server state.
8. Capture the API-to-outbox-to-consumer trace relation; repeat with no/malformed context and confirm processing semantics unchanged.

Reference: event-svc. Run cargo test -p event-svc --test postgres -- --ignored only on the isolated database. Broker end-to-end, publish/commit crash injection, acknowledgement-loss and DLQ replay still require actual owner-supervised evidence. No result or screenshot is prefilled. Do not delete stored poison input to fake success; retain bounded evidence and a correction/replay decision.