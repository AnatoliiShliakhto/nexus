-- [DDL: tb:account]
--DEFINE SEQUENCE account BATCH 1000 START 1000 TIMEOUT 5s;
DEFINE TABLE account SCHEMAFULL PERMISSIONS FOR DELETE WHERE id != account:root; -- PERMISSIONS for delete WHERE id.id() >= 1000;
-- ID: <int> sequence::nextval("account") -- I’m thinking it over?!
DEFINE FIELD username     ON account TYPE string;
DEFINE FIELD password     ON account TYPE option<string> PERMISSIONS FOR SELECT NONE;
DEFINE FIELD status       ON account TYPE string DEFAULT 'ACTIVE' ASSERT $value in ['ACTIVE', 'INACTIVE', 'ARCHIVED'];
DEFINE FIELD allowed_ips  ON account TYPE array<bytes> DEFAULT [];
DEFINE FIELD totp_enabled ON account TYPE bool DEFAULT false;
DEFINE FIELD totp_secret  ON account TYPE option<string> PERMISSIONS FOR SELECT NONE;
DEFINE INDEX idx_username ON account FIELDS username UNIQUE;
DEFINE INDEX idx_status   ON account FIELDS status;

-- [EVENTS]
DEFINE EVENT IF NOT EXISTS audit ON account WHEN $before != $after THEN {
    fn::audit_logger($before, $after)
};

-- [INIT]
UPSERT account:root SET username = "admin", password = crypto::argon2::generate("");