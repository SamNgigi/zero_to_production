# TODOS

### Database Migrations

Want to implement everything from memory as best as I can<br>

Here's a high-level TODO of the tasks that need to be completed
- [x] Add `status` column to `subscriptions` table
    - [x] Generate migration file
    - [x] Write migration script (_allowing null initially_)
    - [x] Run migration
    - [x] Run tests to confirm everything still working as expected
    - [x] Run migration on production db (_Can only be done in axum branch because of the `fly.toml` filed required_)
- [ ] Update `src/routes/subscriptions.rs` `insert_subcriber` with default `status`
    - [ ] Run tests
    - [ ] Deploy updated application.
- [ ] Backfill `status` column with default value "confirmed".
    - [ ] Generate migration file
    - [ ] Write migration script (_mark as `NOT NULL`_)
    - [ ] Run migration
    - [ ] Run tests to confirm everything still working as expected
    - [ ] Run migration on production db (_Can only be done in axum branch because of the `fly.toml` filed required_)
- [ ] Add `subscription_tokens` table
    - [ ] Generate migration file
    - [ ] Write migration script
    - [ ] Run migration
    - [ ] Run tests to confirm everything still working as expected
    - [ ] Run migration on production db (_Can only be done in axum branch because of the `fly.toml` filed required_)
