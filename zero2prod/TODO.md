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
  - [ ] Password Verification - Naive Approach
  - [ ] Password Storage
  - [ ] Do Not Block The Async Executor
  - [ ] User Enumeration
- [ ] Login Form

