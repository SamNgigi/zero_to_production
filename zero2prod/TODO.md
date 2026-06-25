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
  - [ ] Password Storage.
    - [x] Using `sha3` for a cryptographic hash for getting a `password_hash`.
      - [x] Generate migration to update `password` column in `users` table to `password_hash`. 
      - [x] Update `validate_credentials` to generate a `password_hash` from `credentials.password` using `sha3`.
      - [x] Update `validate_credentials` to query on `password_hash` instead of `password`.
      - [x] Add a `TestUser` struct with `generate` and `store` methods.
      - [x] Add `test_user` field to `TestApp` and replace `add_test_user` and `test_user` functions with functionality  
         provided by the `test_user` field in `TestApp`.
    - [ ] Argon2.
      - [ ] Config.
      - [ ] Salting.
      - [ ] PHC Format String.
      - [ ] Update test.
  - [ ] Do Not Block The Async Executor.
  - [ ] User Enumeration.
- [ ] Login Form.

