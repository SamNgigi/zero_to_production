# TODOS

### Securing Our API

Mostly just coding sections

- [ ] Password-based authentication.
  - [x] Basic Authentication.
    - [x] Add `request_mission_authorization_are_reject` test.
    - [x] Add `basic_authentication` function to `newsletters.rs`.
    - [x] Update `publish_newsletter` with `basic_authentication` call.
    - [x] Add `AuthError` variant to `PublishError` enum and add corresponding match for `StatusCode::UNAUTHORIZED`.
    - [x] Implement `ResponseError`'s `error_response` function for `PublishError` adding appropriate header value.
    - [x] Update `app.post_newsletter()` with placeholder/dummy username and password.
  - [x] Password Verification - Naive Approach.
      - [x] Add users table.
        - [x] Add `create_users_table` migration.
        - [x]  Define `users` table.
      - [x] Add `validate_credentials` function to `newsletter.rs`.
      - [x] Update `publish_newsletter` with call to `validate_credentials`.
      - [x] Add tracing instrumentation to `publish_newsletter`.
      - [x] Add `add_test_user` function to `test/api/newsletter.rs`.
      - [x] Update `spawn_app` to call `add_test_user`.
      - [x] Add `test_user` method to `TestApp`.
      - [x] Update `post_newsletters` method to user `test_user` credentials.
  - [x] Password Storage.
    - [x] Using `sha3` for a cryptographic hash for getting a `password_hash`.
      - [x] Generate migration to update `password` column in `users` table to `password_hash`. 
      - [x] Update `validate_credentials` to generate a `password_hash` from `credentials.password` using `sha3`.
      - [x] Update `validate_credentials` to query on `password_hash` instead of `password`.
      - [x] Add a `TestUser` struct with `generate` and `store` methods.
      - [x] Add `test_user` field to `TestApp` and replace `add_test_user` and `test_user` functions with functionality  
         provided by the `test_user` field in `TestApp`.
    - [x] Argon2.
      - [x] Config: Add the `argon2` as a dependency and initialize `hasher` in `validate_credentials`
      - [x] Salting.
        - [x] Add migration to add `salt` column to `users` table
        - [x] Update query in `validate_credentials` to return `user_id`, `password_hash` & `salt`
        - [x] Use `hasher` to generate a password hash from extracted `credentials.password` + `salt`
      - [x] PHC Format String.
        - [x] Use argon2's `PasswordHash` to get PHC formated string from stored `expected_password_hash`
        - [x] Use argon2's `PasswordVerifer`'s `verify_password` to do the equality check
        - [x] Drop the `salt` column
      - [x] Update test: Use argon2's `password_hash::SaltString` to generate a `salt` & hash `test_user`'s password
    - [x] Do Not Block The Async Executor.
      - [x] Add `.with_span_events(FmtSpan::CLOSE)` to `fmt::layer` of `telemetry.rs` module
      - [x] Add `get_stored_credentials` 
      - [x] Inspect how long `Argon2::default().verify_password` takes adding a `tracing::info!` to check time elapsed
      - [x] Add `verify_password_hash` and run it in  `tokio::task::spawn_blocking`
      - [x] Add helper `spawn_blocking_with_tracing` in `src/telemery.rs`.
    - [x] User Enumeration.
      - [x] Add `non_existent_user_is_rejected` test.
      - [x] Add `invalid_user_password_is_rejected` test.
      - [x] Add default `expected_password_hash` and set default `user_id` to `None` pending updated from retrieved query.
- [ ] Login Form.

