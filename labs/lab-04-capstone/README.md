# L4 — Desktop client, API and event pipeline
Outcomes O4/O5/O6/O7; lessons30–34. Completion means a real GUI observing authoritative API state and an event handoff while preserving local undo/restart semantics.

1. Finish L2/L3 environment and reference unit checks. Implement the L4 starter pure proposal.
2. Install the platform's officially documented Tauri requirements on your learner machine. Use the standalone Tauri manifest; the vanilla frontend needs no Node framework.
3. Generate a demo JWT in the current shell, export it as COURSE_TOKEN and launch the API, event consumer and local broker.
4. Run the qualified Tauri CLI in crates/tauri-app: cargo tauri dev. Add/toggle CPU panel; Undo and Redo. Restart and undo restored history.
5. In the GUI submit order-1 with2units. Run the finite outbox publisher. Observe the display event, then explicitly Refresh order and confirm fulfilled via the tenant API.
6. Local Undo must alter only panel state. Refresh the order again and prove that fulfillment remains unchanged.
7. Disconnect NATS: show the observer gap/reconciliation status; query API after reconnect. Stop the API: show the bounded HTTP error without corrupting local history.
8. Open two model instances against a temporary snapshot and observe stale CAS refusal. Invalid/corrupt snapshot must refuse startup without overwriting original bytes.
9. Capture UI accessibility/keyboard behavior, actual GUI build identity, restart evidence and the trace relation. Record unresolved platform or capability issues.

Reference: desktop-core and tauri-app. The observer emits refresh-required markers, never raw order payloads to the UI. No live broker tenant/security contract is claimed by the loopback fixture. Kafka and egui remain optional appendices. No GUI, database, broker or media execution occurred during authoring.