# Reference solution map
Read after attempting the starter. These are authored solutions; no execution result is claimed.

L1 starter: use values.iter().try_fold(0_u64, |sum,value| sum.checked_add(*value).ok_or("overflow")). Complete layout/criterion solution: crates/dop-bench/src/lib.rs and benches/layout.rs.

L2 starter: return MetricColumns::from_batch(batch,tenant). Complete API/middleware/PostgreSQL solution: crates/axum-api, including boundary and explicit ignored storage tests.

L3 starter: refuse attempt zero; Committed or Duplicate => Ack; Invalid => DeadLetter; TransientFailure when attempt<5 => Retry, otherwise DeadLetter. Complete durable solution: crates/event-svc apply/handle/dead_letter plus API order/outbox transaction. A pure decision test cannot certify those effects.

L4 starter: return desktop_core::propose(saved,command). Complete local model/recovery solution: crates/desktop-core. Real GUI/HTTP/event client: crates/tauri-app. Remote orders remain outside local undo.

The deliberate starter blanks are isolated standalone crates, so the reference workspace can be tested independently. Do not remove its negative cases merely to make a starter pass.