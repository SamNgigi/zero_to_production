### Axum Implementation - Error Handling

Want to implement everything from memory as best as I can<br/>

- [ ] Add `subscribe_fails_if_fatal_database_error_occurs` test for inspecting logs
- [ ] Implement the `SubscribeError` enum add `thiserror` procedural macro
- [ ] Implement `IntoResponse` for `SubscribeError`
- [ ] Implement `error_chain_fmt` for chaining source or failure mode.
