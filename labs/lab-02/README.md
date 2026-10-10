# L2 — Tenant metrics API with full middleware
Outcome O4; lessons19,20,22. Use only an isolated learner course_local database. Compose uses synthetic loopback credentials; no production target is permitted.

1. Follow companion/README.md environment setup. Start learner PostgreSQL and local Jaeger with the declared Compose files.
2. Export COURSE_DATABASE_URL, COURSE_MIGRATION_CONFIRM=course_local and a local random COURSE_JWT_SECRET. Run cargo run -p axum-api --bin migrate. Replay it and inspect schema identity.
3. Implement and test the starter acceptance boundary. Run the reference workspace unit suite and the explicit isolated PostgreSQL tests.
4. Generate the demo token with cargo run -p axum-api --bin token -- demo. Keep it out of files and logs.
5. Run cargo run -p axum-api --bin axum-api. POST fixture.json to /ingest with the token; expect204.
6. GET /query?metric=cpu&from=0&to=60000&width=1000. Predict means3 and8 before reading the response.
7. Change one payload tenant and expect403 with no write. Send a missing/expired token and expect401. Exhaust tenant A quota and confirm a second tenant remains independent.
8. Inspect request IDs, trace fields, compression, readiness outage, WebSocket tenant isolation/lag and graceful shutdown. Reconcile the tail through query after a gap.

Reference: crates/axum-api. SQLx runtime binding is implemented; query-macro offline validation is a separate qualification row. Duplicate metric keys replace values by declared policy. Do not describe them as append-only history. Record commands/responses/schema/trace evidence and rollback observations; no practical run occurred on the authoring host.