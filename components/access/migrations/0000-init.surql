-- [DDL: CONSTANTS]
DEFINE PARAM $CONFIG_SESSION_TTL VALUE 30d;
DEFINE PARAM $CONFIG_GRACE_PERIOD VALUE 30s;
DEFINE PARAM $CONFIG_TOKEN_LEN VALUE 64;

-- [DDL: tb:role]
DEFINE TABLE role SCHEMAFULL;
-- ID: <string> slug
DEFINE FIELD name           ON role TYPE string;
DEFINE FIELD description    ON role TYPE option<string>;

-- [DDL: tb:role_permission]
DEFINE TABLE role_permission SCHEMAFULL;
-- ID: [<record> role, <record> permission]
DEFINE FIELD role       ON role_permission TYPE record<role> VALUE <record> id[0];
DEFINE FIELD permission ON role_permission TYPE record<permission> VALUE <record> id[1];
DEFINE FIELD actions    ON role_permission TYPE array<string> DEFAULT [];

-- [DDL: tb:assignment]
DEFINE TABLE assignment SCHEMAFULL;
-- ID: [<record> organization, <record> account, <record> role]
DEFINE FIELD org            ON assignment TYPE record<organization> VALUE <record> id[0];
DEFINE FIELD account        ON assignment TYPE record<account> VALUE <record> id[1];
DEFINE FIELD role           ON assignment TYPE record<role> VALUE <record> id[2];
DEFINE INDEX idx_account    ON assignment FIELDS account;
DEFINE INDEX idx_role       ON assignment FIELDS role;

-- [DDL: tb:_access_cache]
DEFINE TABLE _access_cache SCHEMAFULL PERMISSIONS NONE;
-- ID: [<record> account, <record> organization]
DEFINE FIELD account        ON _access_cache TYPE record<account> VALUE <record> id[0];
DEFINE FIELD org            ON _access_cache TYPE record<organization> VALUE <record> id[1];
-- policy: { "<permission>": ['CREATE', 'READ', 'UPDATE', 'DELETE'] }
DEFINE FIELD policy         ON _access_cache TYPE object DEFAULT {};
DEFINE FIELD policy.*       ON _access_cache TYPE array<string>;
DEFINE FIELD version        ON _access_cache TYPE int DEFAULT 0;

-- [DDL: tb:_access_cache_meta]
DEFINE TABLE _access_cache_meta SCHEMAFULL PERMISSIONS NONE;
-- ID: [<record> account]
DEFINE FIELD account    ON _access_cache_meta TYPE record<account> VALUE id[0];
DEFINE FIELD version    ON _access_cache_meta TYPE int DEFAULT 1;
DEFINE FIELD synced_at  ON _access_cache_meta TYPE datetime DEFAULT ALWAYS time::now();

-- [DDL: tb:_session]
DEFINE TABLE _session SCHEMAFULL;
DEFINE FIELD account        ON _session TYPE record<account>;
DEFINE FIELD jkt            ON _session TYPE string;
DEFINE FIELD refresh_token  ON _session TYPE string PERMISSIONS FOR select NONE;
DEFINE FIELD previous_token ON _session TYPE option<string> PERMISSIONS FOR select NONE;
DEFINE FIELD expires_at     ON _session TYPE datetime VALUE time::now() + $CONFIG_SESSION_TTL;
DEFINE FIELD last_activity  ON _session TYPE datetime VALUE time::now();
DEFINE FIELD status         ON _session TYPE string DEFAULT 'PENDING' ASSERT $value IN ['PENDING', 'ACTIVE', 'REVOKED', 'EXPIRED'];
DEFINE FIELD permissions    ON _session TYPE object FLEXIBLE VALUE fn::get_account_permissions(account);
DEFINE FIELD ip             ON _session TYPE option<bytes> DEFAULT NONE;
DEFINE FIELD user_agent     ON _session TYPE option<string> DEFAULT NONE;
DEFINE INDEX idx_active_session ON _session FIELDS status, expires_at;
DEFINE INDEX idx_token          ON _session FIELDS refresh_token UNIQUE;

-- [DML: fn:sync_account_access_cache]
DEFINE FUNCTION fn::sync_account_access_cache($account: record<account>) {
    DELETE _access_cache:[$account, NONE]..=[$account, ..];
    DELETE _access_cache_meta:[$account];

    LET $raw_data = SELECT
        (SELECT VALUE id FROM organization WHERE chain CONTAINS $parent.org) AS org_list,
        (SELECT VALUE { perm: permission.id(), actions: actions } FROM role_permission WHERE role = $parent.role) AS perms
    FROM assignment WHERE account = $account;

    IF array::len($raw_data) == 0 {
        UPSERT _access_cache_meta:[$account];
        RETURN true;
    };

    LET $entries = (SELECT org_list AS org, perms FROM $raw_data SPLIT org_list);

    FOR $row in (SELECT org, perms FROM $entries WHERE org != NONE GROUP BY org) {
        LET $policy = $row.perms.flatten().fold({}, |$acc, $p| {
            LET $k = <string> $p.perm;
            LET $existing = $acc[$k] OR [];
            LET $new_entry = object::from_entries([[ $k, ($existing + $p.actions).distinct() ]]);
            $acc + $new_entry;
        });

        IF $policy != {} {
            UPSERT _access_cache:[$account, $row.org] SET policy = $policy;
        };
    };

    UPSERT _access_cache_meta:[$account];
    RETURN true;
};

-- [DML: fn:check_access]
DEFINE FUNCTION fn::check_access(
    $account: record<account>,
    $org: record<organization>,
    $permission: string,
    $action: string
) {
    IF $account == account:root {
        RETURN true;
    };

    LET $meta = {
        LET $existing = (SELECT VALUE version FROM ONLY _access_cache_meta:[$account] LIMIT 1);

        IF $existing == NONE {
            fn::sync_account_access_cache($account);
            { version: 1 };
        } ELSE {
            { version: $existing };
        };
    };

    LET $cache_id = type::record("_access_cache", [$account, $org]);
    LET $existing = SELECT VALUE policy FROM $cache_id;
    LET $policy = IF array::len($existing) > 0 { $existing[0] } ELSE { NONE };

    IF $policy != NONE AND $policy[$permission] != NONE {
        $action IN $policy[$permission]
    } ELSE {
        false
    };
};

-- [DML: fn::get_allowed_org_nodes]
DEFINE FUNCTION fn::get_allowed_org_nodes(
    $account: record<account>,
    $permission: string,
    $action: string
) {
    IF $account == account:root { RETURN SELECT VALUE id FROM organization; };

    LET $meta_id = type::record("_access_cache_meta", [$account]);
    LET $is_synced = SELECT VALUE id FROM $meta_id;

    IF array::len($is_synced) == 0 {
        fn::sync_account_access_cache($account);
    };

    RETURN (
        SELECT VALUE org
        FROM _access_cache:[$account, NONE]..=[$account, ..]
        WHERE policy[$permission] CONTAINS $action
    );
};

-- [DML: fn::get_account_permissions]
DEFINE FUNCTION fn::get_account_permissions($account: record<account>) {
    IF $account == account:root {
        LET $all_perms = SELECT VALUE id.id() FROM permission;
        RETURN object::from_entries(
            $all_perms.map(|$p| [ <string>$p, ['CREATE', 'READ', 'UPDATE', 'DELETE'] ])
        );
    };

    LET $meta_id = type::record("_access_cache_meta", [$account]);
    IF (SELECT id FROM $meta_id LIMIT 1) == [] {
        fn::sync_account_access_cache($account);
    };

    LET $all_policies = SELECT VALUE policy FROM _access_cache:[$account, NONE]..=[$account, ..];
    IF $all_policies.len() == 0 { RETURN {} };

    $all_policies.fold({}, |$acc, $policy| {
        LET $shared = $acc.keys().intersect($policy.keys());

        LET $merged_shared = object::from_entries(
            $shared.map(|$k| [
                $k,
                ($acc[$k] + $policy[$k]).distinct()
            ])
        );

        $acc + $policy + $merged_shared;
    });
};

-- [DML: fn::auth_with_credentials]
DEFINE FUNCTION fn::auth_with_credentials(
    $login: string,
    $password: string,
    $jkt: string,
    $ip: option<bytes>,
    $ua: option<string>
) {
    LET $account_id = type::record('account', $login);

    LET $account = (
        SELECT id, password, totp_enabled, allowed_ips
        FROM ONLY $account_id
        WHERE status = 'ACTIVE'
    );

    RETURN IF $account.id AND crypto::argon2::compare($account.password, $password) {
        IF array::len($account.allowed_ips) > 0 AND !($account.allowed_ips CONTAINS $ip) {
                RETURN { status: "Err", code: "IP_RESTRICTED" }
        };

        LET $refresh_token = rand::id($CONFIG_TOKEN_LEN);
        LET $status = IF $account.totp_enabled { 'PENDING' } ELSE { 'ACTIVE' };

        LET $_session = (
            CREATE _session SET
                account = $account.id,
                jkt = $jkt,
                refresh_token = crypto::sha256($refresh_token),
                status = $status,
                ip = $ip,
                user_agent = $ua
        )[0];

        {
            status: "Ok",
            data: {
                session_id: $_session.id.id(),
                account: $account.id.id(),
                refresh_token: $refresh_token,
                permissions: $_session.permissions
            }
        }
    } ELSE {
        {
            status: "Err",
            code: "INVALID_CREDENTIALS"
        }
    };
};

-- [DML: fn::auth_refresh]
DEFINE FUNCTION fn::auth_refresh(
    $id: string,
    $refresh_token: string,
    $jkt: string,
    $ip: option<bytes>,
    $ua: option<string>
) {
    LET $session_id = type::record('_session', $id);
    LET $_session = (
        SELECT *, account.allowed_ips AS account_ips
        FROM ONLY $session_id
        WHERE status == 'ACTIVE' AND expires_at > time::now()
    );

    RETURN IF $_session.id {
        IF array::len($_session.account_ips) > 0 AND !($_session.account_ips CONTAINS $ip) {
            UPDATE $_session.id SET status = 'REVOKED';
            RETURN { status: "Err", code: "IP_RESTRICTED" }
        };

        LET $current_token_hash = crypto::sha256($refresh_token);

        IF $_session.refresh_token == $current_token_hash {
            LET $new_token = rand::id($CONFIG_TOKEN_LEN);

            UPDATE $_session.id SET
                previous_token = $_session.refresh_token,
                refresh_token = crypto::sha256($new_token),
                jkt = $jkt,
                ip = $ip,
                user_agent = $ua;

            {
                status: "Ok",
                data: {
                    session_id: $_session.id.id(),
                    account: $_session.account.id.id(),
                    refresh_token: $new_token,
                    permissions: $_session.permissions
                }
            }
        }
        ELSE IF $_session.previous_token == $current_token_hash
             AND (time::now() - $_session.last_activity) < $CONFIG_GRACE_PERIOD
        {
            LET $new_token = rand::id($CONFIG_TOKEN_LEN);

            UPDATE $_session.id SET
                refresh_token = crypto::sha256($new_token),
                jkt = $jkt,
                ip = $ip,
                user_agent = $ua;

            {
                status: "Ok",
                data: {
                    session_id: $_session.id.id(),
                    account: $_session.account.id.id(),
                    refresh_token: $new_token,
                    permissions: $_session.permissions,
                    allowed_ips: $_session.account_ips
                }
            }
        }
        ELSE {
            UPDATE $_session.id SET status = 'REVOKED';
            { status: "Err", code: "SECURITY_VIOLATION" }
        }
    } ELSE {
        { status: "Err", code: "SESSION_EXPIRED" }
    };
};

-- [EVENTS]
DEFINE EVENT audit ON role WHEN $before != $after THEN {
    fn::audit_logger($before, $after)
};

DEFINE EVENT audit ON role_permission WHEN $before != $after THEN {
    fn::audit_logger($before, $after)
};

DEFINE EVENT audit ON assignment WHEN $before != $after THEN {
    fn::audit_logger($before, $after)
};

DEFINE EVENT sync_on_assignment ON TABLE assignment WHEN $event = 'CREATE' OR $event = 'DELETE' THEN {
    LET $account = $after.account OR $before.account;
    DELETE _access_cache:[$account, NONE]..=[$account, ..];
    DELETE type::record("_access_cache_meta", [$account])
};

DEFINE EVENT sync_on_role_change ON TABLE role_permission WHEN $event = 'UPDATE' THEN {
    LET $affected = (SELECT VALUE account FROM assignment WHERE role = $after.role);
    DELETE _access_cache WHERE account IN $affected;
    DELETE _access_cache_meta WHERE account IN $affected;
};

-- [DML: fn::auth_revoke]
DEFINE FUNCTION fn::auth_revoke(
    $id: string,
    $ip: option<bytes>
) {
    LET $session_id = type::record('_session', $id);

    LET $_session = (SELECT id FROM ONLY $session_id WHERE status != 'REVOKED');

    IF $_session.id {
        UPDATE $_session.id SET
            status = 'REVOKED',
            refresh_token = NONE,
            previous_token = NONE,
            ip = $ip;
    };

    RETURN { status: "Ok" };
};

-- DEFINE EVENT sync_owner_role ON TABLE permission WHEN $event = 'CREATE' THEN {
--     LET $rp_id = type::record("role_permission", [role:owner, $after.id]);
--     UPSERT $rp_id SET actions = ['CREATE', 'READ', 'UPDATE', 'DELETE'];
-- };

-- UPSERT organization:root SET name = "Global Headquarters", parent = NONE;
-- UPSERT role:owner SET name = "Super Administrator", description = "Full administrative control over the platform.";
-- LET $permissions = SELECT VALUE id FROM permission;
-- FOR $p_id IN $permissions {
--     LET $rp_id = type::record("role_permission", [role:owner, $p_id]);
--     UPSERT $rp_id SET actions = ['CREATE', 'READ', 'UPDATE', 'DELETE'];
-- };
-- UPSERT assignment:[organization:root, account:root, role:owner];