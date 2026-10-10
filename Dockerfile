# Proposed API-only learner build; authored and NOT executed.
# Image tags are inherited authoring targets, not verified immutable digests.
FROM rust:1.90.0-slim-bookworm AS build
WORKDIR /course

# Keep the sealed workspace manifests and actual target entry files intact.
# Cargo resolves this workspace even when only the API binary is selected.
COPY ["Cargo.toml", "rust-toolchain.toml", "./"]
COPY ["crates/shared-kernel/Cargo.toml", "crates/shared-kernel/"]
COPY ["crates/shared-kernel/src/lib.rs", "crates/shared-kernel/src/"]
COPY ["crates/aspects/Cargo.toml", "crates/aspects/"]
COPY ["crates/aspects/src/lib.rs", "crates/aspects/src/"]
COPY ["crates/audit-macro/Cargo.toml", "crates/audit-macro/"]
COPY ["crates/audit-macro/src/lib.rs", "crates/audit-macro/src/"]
COPY ["crates/axum-api/Cargo.toml", "crates/axum-api/"]
COPY ["crates/axum-api/src/lib.rs", "crates/axum-api/src/main.rs", "crates/axum-api/src/"]
COPY ["crates/axum-api/src/bin/migrate.rs", "crates/axum-api/src/bin/token.rs", "crates/axum-api/src/bin/"]
COPY ["crates/axum-api/migrations/0001_course.sql", "crates/axum-api/migrations/"]
COPY ["crates/dop-bench/Cargo.toml", "crates/dop-bench/"]
COPY ["crates/dop-bench/src/lib.rs", "crates/dop-bench/src/"]
COPY ["crates/dop-bench/benches/layout.rs", "crates/dop-bench/benches/"]
COPY ["crates/event-svc/Cargo.toml", "crates/event-svc/"]
COPY ["crates/event-svc/src/lib.rs", "crates/event-svc/src/main.rs", "crates/event-svc/src/"]
COPY ["crates/desktop-core/Cargo.toml", "crates/desktop-core/"]
COPY ["crates/desktop-core/src/lib.rs", "crates/desktop-core/src/"]
COPY ["crates/extras-solutions/Cargo.toml", "crates/extras-solutions/"]
COPY ["crates/extras-solutions/src/lib.rs", "crates/extras-solutions/src/"]

# The sealed source has no Cargo.lock. A later admitted qualification build
# must retain and review its generated lock before a separate locked revision.
# Select the compiler explicitly; formatter/linter components are not needed.
RUN cargo +1.90.0 build --release -p axum-api --bin axum-api

FROM gcr.io/distroless/cc-debian12:nonroot AS runtime
COPY --from=build /course/target/release/axum-api /app
USER 65532:65532
ENV COURSE_BIND=0.0.0.0:3000
EXPOSE 3000
ENTRYPOINT ["/app"]
