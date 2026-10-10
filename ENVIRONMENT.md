# Proposed learner environment — qualification required

Baseline: stable Rust 1.90.0, edition2021, native x86_64 learner host. The Rust release exists in the official release record. Direct crate versions below were checked against maintainer-published API pages on2026-10-10. This establishes API reference provenance, not a supported compiled dependency closure. No resolution, download, native build, lock generation or execution happened in authoring.

| Surface | Proposed pin | Official reference | Qualification still needed |
|---|---|---|---|
| Rust | 1.90.0 | https://blog.rust-lang.org/2025/09/18/Rust-1.90.0/ | Record rustc -Vv, target, linker and actual build |
| serde / JSON | 1.0.228 / 1.0.145 | https://docs.rs/serde/1.0.228/serde/ and https://docs.rs/serde_json/1.0.145/serde_json/ | Resolve features and roundtrip fixtures |
| thiserror / anyhow | 2.0.17 / 1.0.100 | https://docs.rs/thiserror/2.0.17/thiserror/ and https://docs.rs/anyhow/1.0.100/anyhow/ | Compile error conversions |
| tokio | 1.47.1 | https://docs.rs/tokio/1.47.1/tokio/ | Runtime, cancellation, broadcast and shutdown |
| axum | 0.8.6 | https://docs.rs/axum/0.8.6/axum/ | Extractors, middleware and real WebSocket integration |
| tower / tower-http | 0.5.3 / 0.6.6 | https://docs.rs/tower/0.5.3/tower/ and https://docs.rs/tower-http/0.6.6/tower_http/ | Readiness/layer-order and response tests |
| tower_governor | 0.8.0 | https://docs.rs/tower_governor/0.8.0/tower_governor/ | Dependency compatibility and clock-sensitive quota proof |
| tracing / subscriber | 0.1.41 / 0.3.23 | https://docs.rs/tracing/0.1.41/tracing/ and https://docs.rs/tracing-subscriber/0.3.23/tracing_subscriber/ | Recorder lifecycle and sensitive-field exclusion |
| tracing-opentelemetry | 0.32.0 | https://docs.rs/tracing-opentelemetry/0.32.0/tracing_opentelemetry/ | Parent/context and error conversions |
| opentelemetry / sdk / otlp | 0.31.0 | https://docs.rs/opentelemetry/0.31.0/opentelemetry/ and https://docs.rs/opentelemetry_sdk/0.31.0/opentelemetry_sdk/ and https://docs.rs/opentelemetry-otlp/0.31.0/opentelemetry_otlp/ | OTLP receiver compatibility, shutdown and export failures |
| metrics / prometheus exporter | 0.24.6 / 0.18.3 | https://docs.rs/metrics/0.24.6/metrics/ and https://docs.rs/metrics-exporter-prometheus/0.18.3/metrics_exporter_prometheus/ | HTTP feature closure, actual recorder/readout |
| sqlx | 0.8.6 | https://docs.rs/sqlx/0.8.6/sqlx/ | Migration, array encoding, transactions and isolated DB tests |
| jsonwebtoken | 9.3.1 | https://docs.rs/jsonwebtoken/9.3.1/jsonwebtoken/ | Signature/expiry/issuer/audience negative cases |
| async-nats | 0.50.0 | https://docs.rs/async-nats/0.50.0/async_nats/ | MSRV/transitive closure, durable publish/pull/ack and headers |
| futures-util | 0.3.31 | https://docs.rs/futures-util/0.3.31/futures_util/ | Stream adapter compile |
| hecs / wide | 0.10.5 / 0.7.33 | https://docs.rs/hecs/0.10.5/hecs/ and https://docs.rs/wide/0.7.33/wide/ | ECS query and equal-work stable SIMD |
| criterion | 0.5.1 | https://docs.rs/criterion/0.5.1/criterion/ | Raw benchmark provenance; no prewritten timing |
| syn / quote | 2.0.106 / 1.0.47 | https://docs.rs/syn/2.0.106/syn/ and https://docs.rs/quote/1.0.47/quote/ | Macro expansion/early-return/question-mark tests |
| sha2 | 0.10.9 | https://docs.rs/sha2/0.10.9/sha2/ | Stable semantic envelope fingerprint |
| rkyv | 0.8.18 | https://docs.rs/rkyv/0.8.18/rkyv/ | Checked archive API and length/nonfinite/overflow tests |
| sled | 0.34.7 | https://docs.rs/sled/0.34.7/sled/ | Learner-local CAS/flush/restart/corrupt-data limits |
| reqwest | 0.12.23 | https://docs.rs/reqwest/0.12.23/reqwest/ | Fixed local URL, timeout, redirects disabled |
| Tauri / tauri-build | 2.8.5 / 2.4.1 | https://docs.rs/tauri/2.8.5/tauri/ and https://docs.rs/tauri-build/2.4.1/tauri_build/ | CLI/native/webview/capability/CSP integration |
| eframe | 0.32.3 | https://docs.rs/eframe/0.32.3/eframe/ | Standalone native appendix build/run |
| rdkafka | 0.38.0 | https://docs.rs/rdkafka/0.38.0/rdkafka/ | librdkafka/CMake/linker and offset/rebalance boundaries |

Quote1.0.47 and tracing-subscriber0.3.23 use directly accessible official API versions; earlier candidate guesses1.0.40/0.3.20 were removed before freeze. No claim that these are latest or best versions follows from selection. Transitive MSRV may force a bounded toolchain or pin change during qualification; record that as a new candidate rather than silently changing a reviewed manifest.

Containers proposed for a local learner exercise:

| Service | Tag | Provenance and remaining checks |
|---|---|---|
| PostgreSQL | postgres:17.6-bookworm | https://www.postgresql.org/docs/17/release-17-6.html ; image pull/digest/platform unperformed |
| NATS | nats:2.11.8-alpine | https://github.com/nats-io/nats-server/releases/tag/v2.11.8 ; image/config/persistence qualification pending |
| Jaeger | jaegertracing/all-in-one:1.68.0 | https://github.com/jaegertracing/jaeger/releases/tag/v1.68.0 ; OTLP receiver and actual image digest pending |
| Builder | rust:1.90.0-slim-bookworm | Rust release above; image presence/digest/native links pending |
| API runtime | gcr.io/distroless/cc-debian12:nonroot | https://github.com/GoogleContainerTools/distroless ; libc/libs/platform and signature/digest pending |

Mutable image tags do not establish reproducibility. First qualification pulls/inspects actual images and records immutable digests. The Dockerfile is a proposed local exercise and is not a deployable production contract.

Windows needs official Rust MSVC linker prerequisites and Tauri's C++ tools/WebView2. Linux/macOS need the respective official webview/native prerequisites. See https://v2.tauri.app/start/prerequisites/ . The static vanilla frontend uses no Node framework. The Tauri CLI version compatible with this manifest must be selected and recorded before native execution; no global install is performed by this folder.

The root workspace excludes GUI/Kafka appendices and intentionally unfinished standalone starters. Its reference tests are separate from exercise tests. Every PostgreSQL integration test is explicitly ignored by default and requires a throwaway migrated loopback course_local DB. The explicit migrate binary checks parsed host/database and confirmation. Never reuse unrelated credentials, stores or another workspace's database.

Optional tooling uses official routes: rust-analyzer https://rust-analyzer.github.io/book/installation.html ; nextest https://nexte.st/docs/installation/pre-built-binaries/ ; assembly inspection https://github.com/pacak/cargo-show-asm ; advisories https://rustsec.org/ ; dependency policy https://embarkstudios.github.io/cargo-deny/ . Record installed versions and exact outputs. No cargo-audit/cargo-deny pass or signed artifact exists here.

Material execution is a later resource-admitted existing-service responsibility. No replacement runner, package manager workflow or repository CI was introduced. This environment file is learner/reviewer instruction, not an execution receipt.
## Container input closure for this source revision

The exact proposed Dockerfile and .dockerignore are integrated from the independent platform proposal. The file-only allowlist contains 24 unchanged inputs: root workspace/toolchain, all eight workspace manifests and real primary target entries, the explicitly declared layout benchmark, API migration/token binaries and 0001_course.sql. Only axum-api plus shared-kernel/aspects/audit-macro is selected for compilation. No placeholder source or replacement workspace was created.

The bounded source review found no selected local build.rs or external mod/include/path input beyond migrate.rs's sqlx::migrate! directory. Tauri's build.rs belongs to an excluded package. Tests, examples, GUI/frontend, standalone starters, appendices, guides, fixture JSON, Compose/config files, local stores, environment files and credentials are not admitted to the proposed API context. Future new compile inputs require a new exact closure review.

No matcher/context archive or build was executed, so these are source expectations. Qualification must observe the actual context file list and excluded canaries. There is no Cargo.lock; the proposed unrun build can resolve dependencies only in a later admitted qualification. A reviewed workspace lock must then be admitted and copied explicitly by a new configuration revision before --locked is used. Base-image tags, digests, platform/native linkage, notices, signature and startup remain unqualified.

This review used primary references: [Docker context and ignore rules](https://docs.docker.com/build/concepts/context/#dockerignore-files), [Cargo workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html), [Moby archive traversal](https://github.com/moby/go-archive/blob/main/archive.go) and the [official Rust image template](https://github.com/rust-lang/docker-rust/blob/master/Dockerfile-slim.template). Documentation and manual source inspection do not substitute for the matching qualified build/context evidence.
