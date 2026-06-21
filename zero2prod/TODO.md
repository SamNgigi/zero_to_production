# TODOS

### Securing Our API

Mostly just coding sections

- [ ] Password-based authentication
  - [ ] Basic Authentication
    - [ ] Add `request_mission_authorization_are_reject` test
    - [ ] Add `basic_authentication` function to `newsletters.rs`
    - [ ] Update `publish_newsletter` with `basic_authentication` call.
    - [ ] Add `AuthError` variant to `PublishError` enum and add corresponding match for `StatusCode::UNAUTHORIZED`
    - [ ] Implement `ResponseError`'s `error_response` function for `PublishError` adding appropriate header value.
    - [ ] Update `app.post_newsletter()` with placeholder/dummy username and password.
  - [ ] Password Verification - Naive Approach
  - [ ] Password Storage
  - [ ] Do Not Block The Async Executor
  - [ ] User Enumeration
- [ ] Login Form

