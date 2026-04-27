# TODOS

### Skeleton And Principles For A Maintainable Test Suite 

Here's a high-level TODO of the tasks that need to be completed for the axum branch
- [x] Create `api` submodule in tests with;
    - [x] `main.rs, helpers.rs, health_check.rs, subscriptions.rs` files
    - [x] breakdown original `health_check.rs` into the relevant files. Move or Delete original
    - [x] Run tests to ensure everything is working as before
- [ ] Refactor `src/main.rs` config setup and `run` into a `build` function
- [ ] Refactor `tests/api/helpers.rs`'s `spawn_app` to make use of `build` for config setup and running the app
    - [ ] Refactor `connection_pool`/`db_pool` creation to separate `get_connection_pool`
    - [ ] Add `Application` struct to implementation to be able to return `port` and `server` after app is built.
- [x] Refactor subscriptions test to extract out duplicate client request code into its own `TestApp::post_subscriptions()` method
