# Rust DOP/AOP course companion — authored source edition
Learn to explain data layout, compose visible cross-cutting joins, and recover bounded API/event/desktop workflows. This folder contains code, synthetic fixtures, labs, EXTRA practice and reference solutions. Exclusive lesson narration/media remain outside the companion.

This public source edition belongs to [Cataldir/rust-dop-aop-course](https://github.com/Cataldir/rust-dop-aop-course), repository ID 1413720619. It contains authored, manually reviewed learner source. No clean clone, installation, compilation or test has been observed. No code LICENSE or open-source reuse grant has been adopted. Third-party crates, images and tools retain their own terms; their source and images are not bundled here.

The intended learner source route follows. The installation and execution commands are proposed instructions, not observed results:
    git clone https://github.com/Cataldir/rust-dop-aop-course
    cd rust-dop-aop-course
    git rev-parse HEAD
Record the actual published commit and reviewed lock/environment before installation. This source edition contains no clone/install proof; material execution requires admission.

## Install and run on a qualified learner machine
Proposed stable Rust1.90.0; exact direct pins in Cargo.toml, platform requirements in ENVIRONMENT.md. No generated Cargo.lock or build/test result is included. First qualification resolves and reviews the lock; subsequent reproducible runs use --locked.

    cargo test --workspace
    cargo clippy --workspace --all-targets -- -D warnings
    cargo fmt --all --check
    cargo bench -p dop-bench --bench layout

Standalone starter tests intentionally fail until the learner implements their blanks. Run their explicit manifest commands in each lab. GUI/Kafka appendices have separate manifests and native prerequisites. Reference workspace default tests skip explicitly ignored PostgreSQL observations; run those separately against isolated course_local.

From companion root on your learner host:
    docker compose up -d
    docker compose -f docker-compose.observability.yml up -d
Set COURSE_DATABASE_URL to postgres://course:course-local-only@127.0.0.1:5433/course_local and COURSE_MIGRATION_CONFIRM to course_local. The password is synthetic teaching data. Set a local random COURSE_JWT_SECRET of at least32bytes; never commit or log it.
    cargo run -p axum-api --bin migrate
    cargo run -p axum-api --bin token -- demo
    cargo run -p axum-api --bin axum-api

Use the returned token only in the current shell's Authorization header. API binds loopback by default; any broader listener configuration is an explicit learner deployment decision. Query samples and commands are in labs/lab-02. Prometheus-format metrics: loopback9000; Jaeger UI: loopback16686.

For the event handoff:
    cargo run -p event-svc -- publish
    cargo run -p event-svc -- consume
Publish is a finite32-row dispatcher; repeat it for new outbox rows. It does not start a scheduler. L3 describes broker/replay/DLQ checks.

For Tauri, qualify its CLI/platform prerequisites, set COURSE_TOKEN in the launching shell, then from crates/tauri-app run cargo tauri dev. Vanilla static frontend requires no Node framework. Use L4's real API/broker workflow. Optional standalone egui/Kafka manifests live under appendix/.

## Extend safely
Start with a local branch and one changed outcome. Add the negative case first: cross-tenant input, stale UI revision, changed-content event key or missing SIMD tail. Keep stable wire IDs/version and the named effect joins. Use EXTRA.md hints before reference-solutions.md. Preserve raw experiment outputs under ignored results/, and record commit/environment/lock hashes.

All fixtures are newly authored synthetic teaching data. No production dataset, institution/professor asset, FIAP material, voice recording or exclusive paid-course text is included. Third-party crates/images retain their own licenses; this candidate adds no blanket redistribution permission.

## Qualification status
Authored code/fixtures/tests exist. Compiler, dependency resolution, tests, benches, PostgreSQL migrations, JetStream, Tauri/egui/Kafka, images, telemetry and security tools have **not run** on the authoring host. Resource and environment qualification remain pending. release-checklist.md records the later required observations.
## API container source revision
The proposed Dockerfile builds only axum-api and copies only its executable into the runtime stage. The companion root is the intended context. The exact-file .dockerignore admits 24 unchanged source/manifest inputs; no whole source directory is admitted. Workspace discovery files, the explicit benchmark, API auxiliary binaries and SQL include are retained without selecting them for execution. No application or dependency change accompanied this revision.

No actual context listing, canary check, lock resolution, image pull/build or startup ran. Current image tags and native links remain unqualified. The absent Cargo.lock must be resolved and reviewed on an admitted environment before a later revision explicitly includes it and uses --locked. Keep runtime secrets outside the build context and use host-loopback port publication for the local learner exercise. ENVIRONMENT.md and release-checklist.md state the remaining observations.
