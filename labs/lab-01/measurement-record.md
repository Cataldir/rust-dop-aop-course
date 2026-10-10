# L1 measurement record — learner fills actual observations

This is an empty evidence form, not benchmark output.

- Source commit and changed files:
- Rust version/target and optimizer/target-feature settings:
- OS, CPU, memory and load/thermal conditions:
- Cargo.lock hash and dependency review:
- Variant, operation and fixture size:
- Equality/rounding validation result and raw path:
- Criterion command and raw report path:
- Estimate/uncertainty copied from actual output:
- Conversion/allocation included or excluded, with rationale:
- Repeat/change-control observations:
- Profile command/tool version and actual artifact:
- Interpretation and limitations:

After official profiler installation/host permission is qualified, one optional route is:
    cargo flamegraph -p dop-bench --bench layout -o results/layout-profile.svg -- --bench
Maintainer instructions: https://github.com/flamegraph-rs/flamegraph . Set bench debug information on a learner branch when symbols are needed. Its Linux Rust1.90/lld guidance requires recording linker/profile settings; do not silently alter the benchmark comparison. Windows/macOS use their documented native backend. No profiler was installed or run here.

Read the flame graph width as sampled stack frequency, not a left-to-right timeline. Use it to select a question, then measure that question with the controlled benchmark. Keep raw output and the original baseline before an optimization.