# TODOS

### Database Migrations

Want to implement everything from memory as best as I can<br>

Here's a high-level TODO of the tasks that need to be completed
- [x] Add `status` column to `subscriptions` table
    - [x] Generate migration file
    - [x] Write migration script (_allowing null initially_)
    - [x] Run migration
    - [x] Run tests to confirm everything still working as expected
    - [x] Run migration on production db
- [x] Update `src/routes/subscriptions.rs` `insert_subcriber` with default `status`
    - [x] Run tests
    - [x] Deploy updated application.
- [x] Backfill `status` column with default value "confirmed".
    - [x] Generate migration file
    - [x] Write migration script (_mark as `NOT NULL`_)
    - [x] Run migration
    - [x] Run tests to confirm everything still working as expected
    - [x] Run migration on production db
- [x] Add `subscription_tokens` table
    - [x] Generate migration file
    - [x] Write migration script
    - [x] Run migration
    - [x] Run tests to confirm everything still working as expected
    - [x] Run migration on production db
