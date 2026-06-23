# TODOS

### Securing Our API

Mostly just coding sections

- [ ] Password-based authentication
  - [x] Basic Authentication
    - [x] Add `request_mission_authorization_are_reject` test
    - [x] Add `basic_authentication` function to `newsletters.rs`
    - [x] Update `publish_newsletter` with `basic_authentication` call.
    - [x] Add `AuthError` variant to `PublishError` enum and add corresponding match for `StatusCode::UNAUTHORIZED`
    - [x] Implement `ResponseError`'s `error_response` function for `PublishError` adding appropriate header value.
    - [x] Update `app.post_newsletter()` with placeholder/dummy username and password.
  - [ ] Password Verification - Approach
      - [x] Add users table
        - [x] Add `create_users_table` migration.
        - [x]  Define `users` table
      - [x] Add `validate_credentials` function to `newsletter.rs`
      - [x] Update `publish_newsletter` with call to `validate_credentials`
      - [x] Add tracing instrumentation to `publish_newsletter`
      - [x] Add `add_test_user` function to `test/api/newsletter.rs`
      - [x] Update `spawn_app` to call `add_test_user`
      - [x] Add `test_user` method to `TestApp`
      - [x] Update `post_newsletters` method to user `test_user` credentials.
  - [ ] Password Storage
  - [ ] Do Not Block The Async Executor
  - [ ] User Enumeration
- [ ] Login Form

