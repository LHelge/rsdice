DROP INDEX IF EXISTS idx_email_verification_tokens_user_id;
DROP TABLE IF EXISTS email_verification_tokens;

ALTER TABLE users
DROP COLUMN IF EXISTS email_verified,
DROP COLUMN IF EXISTS email;
