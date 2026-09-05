-- Add migration script here
ALTER TABLE issue_delivery_queue 
RENAME COLUMN email TO subscriber_email;
