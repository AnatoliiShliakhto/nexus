-- [DDL: tb:audit]
DEFINE TABLE audit SCHEMAFULL PERMISSIONS NONE;
-- ID: [<record> any, <datetime> timestamp, <uuid> rand::uuid()]
DEFINE FIELD target                 ON audit TYPE record VALUE <record> id[0];
DEFINE FIELD timestamp              ON audit TYPE datetime VALUE <datetime> id[1];
DEFINE FIELD account                ON audit TYPE option<record<account>>;
DEFINE FIELD action                 ON audit TYPE string ASSERT $value IN ['CREATE', 'UPDATE', 'DELETE'];
DEFINE FIELD changes                ON audit TYPE any;
DEFINE INDEX idx_timestamp          ON audit FIELDS timestamp;
DEFINE INDEX idx_account_activity   ON audit FIELDS account, timestamp;

-- [DML: fn:audit_logger]
DEFINE FUNCTION fn::audit_logger(
    $before: any,
    $after: any
) {
    -- 1. CONSTANTS & STATE
    LET $is_create = $before == NONE;
    LET $is_delete = $after == NONE;
    LET $target = IF $is_create { $after.id } ELSE { $before.id };
    LET $excluded = ['created_at', 'updated_at', 'id', 'password', 'chain'];
    IF $target == NONE { RETURN NONE };

    -- 2. CALCULATE CHANGES
    LET $changes = IF $is_create {
        array::filter(array::map(object::entries($after), |$entry|
            IF $entry[0] NOT IN $excluded {
                { path: $entry[0], new_value: $entry[1] }
            }),
            |$c| $c != NONE
        )
    } ELSE IF $is_delete {
        array::filter(array::map(object::entries($before), |$entry|
            IF $entry[0] NOT IN $excluded {
                { path: $entry[0], old_value: $entry[1] }
            }),
            |$c| $c != NONE
        )
    } ELSE {
        array::filter(array::map(value::diff($before, $after), |$d| {
            LET $path = string::slice($d.path, 1);
            IF $path NOT IN $excluded AND $d.op IN ['replace', 'add', 'remove', 'change'] {
                {
                    path: $path,
                    new_value: IF $d.op == 'remove' { NONE } ELSE { $after[$path] },
                    old_value: $before[$path]
                }
            }}),
            |$c| $c != NONE
        )
    };

    -- 3. PERSISTENCE WITH COMPOSITE ID
    IF $target != NONE AND array::len($changes) > 0 {
        LET $audit_id = type::record('audit', [$target, time::now(), rand::uuid()]);

        CREATE $audit_id SET
            action = IF $is_create { 'CREATE' } ELSE IF $is_delete { 'DELETE' } ELSE { 'UPDATE' },
            account  = $auth,
            target = $target,
            changes = $changes,
            timestamp = time::now();
    };
};