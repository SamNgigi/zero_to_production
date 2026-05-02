# TODOS

### Database Migrations

Want to implement everything from memory as best as I can<br>

Here's a high-level TODO of the tasks that need to be completed
- [ ] Add `status` column to `subscriptions` table
    - [ ] Generate migration file
    - [ ] Write migration script
    - [ ] Run migration
    - [ ] Run tests to confirm everything still working as expected
    - [ ] Run migration on production db (_Can only be done in axum branch because of the `fly.toml` filed required_)
- [ ] Add `subscription_tokens` table
    - [ ] Generate migration file
    - [ ] Write migration script
    - [ ] Run migration
    - [ ] Run tests to confirm everything still working as expected
    - [ ] Run migration on production db (_Can only be done in axum branch because of the `fly.toml` filed required_)
