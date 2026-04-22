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