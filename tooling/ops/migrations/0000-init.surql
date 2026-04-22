-- [DDL: CONSTANTS]
DEFINE PARAM $ACTION_CREATE VALUE "CREATE";
DEFINE PARAM $ACTION_READ   VALUE "READ";
DEFINE PARAM $ACTION_UPDATE VALUE "UPDATE";
DEFINE PARAM $ACTION_DELETE VALUE "DELETE";

-- [DDL: SESSION]
--DEFINE ACCESS accounts ON DATABASE TYPE RECORD WITH JWT ALGORITHM EDDSA KEY $public_key;

-- [DDL: tb:component]
DEFINE TABLE component SCHEMAFULL PERMISSIONS NONE;
-- ID: <string> slug
DEFINE FIELD name           ON component TYPE string;
DEFINE FIELD description    ON component TYPE option<string>;
DEFINE FIELD status         ON component TYPE string DEFAULT 'ACTIVE' ASSERT $value IN ['ACTIVE', 'MAINTENANCE', 'DISABLED', 'SYSTEM'];
DEFINE FIELD target         ON component TYPE option<string> ASSERT $value == NONE OR string::starts_with($value, 'http');
DEFINE FIELD route          ON component TYPE option<string> ASSERT $value == NONE OR string::starts_with($value, '/');
DEFINE FIELD protected      ON component TYPE bool DEFAULT true;
DEFINE INDEX idx_route      ON component FIELDS route UNIQUE;
DEFINE INDEX idx_status     ON component FIELDS status;

-- [DDL: tb:migration]
DEFINE TABLE migration SCHEMAFULL PERMISSIONS NONE;
-- ID: [<record> component, <string> version]
DEFINE FIELD component      ON migration TYPE record<component> VALUE <record> id[0];
DEFINE FIELD version        ON migration TYPE string VALUE <string> id[1];
DEFINE FIELD checksum       ON migration TYPE string READONLY;

-- [DDL: tb:permission]
DEFINE TABLE permission SCHEMAFULL PERMISSIONS NONE;
-- ID: <string> slug
DEFINE FIELD name           ON permission TYPE string;
DEFINE FIELD description    ON permission TYPE option<string>;

-- [DML: fn:ensure_component]
DEFINE FUNCTION fn::ensure_component(
    $component: string,
    $name: string,
    $description: option<string>,
    $status: string,
    $target: option<string>,
    $route: option<string>,
    $protected: bool,
) {
    LET $c_id = type::record("component", $component);
    IF $protected {
        LET $p_id = type::record("permission", $component);
        UPSERT $p_id SET name = $name, description = $description;
    };
    RETURN UPSERT $c_id SET name = $name, description = $description, status = $status, target = $target, route = $route;
};

-- [DML: confirm_migration]
DEFINE FUNCTION fn::confirm_migration(
    $component: string,
    $version: string,
    $checksum: string
) {
    LET $s_id = type::record("component", $component);
    LET $m_id = type::record("migration", [$s_id, $version]);
    LET $component_exists = (SELECT id FROM $s_id)[0];

    IF !$component_exists {
        THROW "Component '" + $component + "' not registered";
    };

    LET $existing = (SELECT checksum FROM $m_id)[0];
    IF $existing && $existing.checksum != $checksum {
        THROW "Component '" + $component + "' migration '" + $version + "' checksum mismatch";
    };

    CREATE $m_id SET checksum = $checksum;
};