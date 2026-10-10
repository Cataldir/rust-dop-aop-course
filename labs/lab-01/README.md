# L1 — Benchmark duel: AoS, SoA and ECS
Outcome O2; lessons07,10,11. Deliver a reproducible experiment whose variants perform equal arithmetic, with honest limits. No speed ranking is required.

1. Read Row, Columns and Value. Predict which bytes a value-only scan touches.
2. Implement the standalone starter checked sum. Run cargo test --manifest-path labs/lab-01/starter/Cargo.toml and preserve the overflow case.
3. Run cargo test -p dop-bench. Inspect empty, remainder and equal-work cases.
4. Run cargo run -p dop-bench --example layout and record actual sizes/offsets.
5. Run cargo bench -p dop-bench --bench layout. Keep conversion separate from scans.
6. Repeat one controlled change: an operation that reads timestamp as well as value, or a larger cold payload. Keep the original output.
7. Profile a qualified optimized binary with your OS-approved profiler and explain what the profile supports.

Reference: dop-bench crate; reference-solutions.md. Report actual CPU/OS/compiler, lockfile hash, command, fixture sizes, sample interval, raw artifact paths and interpretation. Record rounding policy for arbitrary floats; the supplied integer-like fixture does not prove every summation order equal. Failure cases: omitted SIMD remainder, work omitted from one variant, conversion included in only one timing, noisy host. Expected evidence remains unobserved until your run.