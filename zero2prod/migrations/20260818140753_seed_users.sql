-- Add migration script here
INSERT INTO users (user_id, username, password_hash)
VALUES (
  '019fcb48-4b42-768f-a117-ea93c2964c81',
  'admin',
  '$argon2id$v=19$m=19456,t=2,p=1$yFIw2eHN2DJARIRpszlqHw$2M1pVj8UZBVT7fW1EN95oc0pPHrq4vfrzSeSSvIKBUc'
);
