### Axum Implementation - Naive Newsletter Delivery

Want to implement everything from memory as best as I can<br/>

- [x] Add relevant tests
    - [x] Add `test/api/newsletters.rs` module
    - [x] `newsletters_are_not_delivered_to_unconfirmed_subscribers`
    - [x] `newsletters_are_delivered_to_confirmed_subscribers`
    - [x] `newsletter_returns_400_for_invalid_data`
- [ ] Naive implementation
    - [x] Add `src/routes/newsletters.rs` module 
    - [x] Implement skeleton `publish_newsletter` handler and update route
    - [x] Add newsletter `BodyContent`
    - [ ] Implement `get_confirmed_subscribers`
    - [ ] Add `PublishError` for error handling
    - [ ] Flesh out final `publish_newsletter` implementation
- [ ] Ensure all tests are green

