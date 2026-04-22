-- [DDL: tb:organization]
DEFINE TABLE organization SCHEMAFULL PERMISSIONS FOR DELETE WHERE id != organization:root;
DEFINE FIELD name       ON organization TYPE string;
DEFINE FIELD parent     ON organization TYPE option<record<organization>> ASSERT $value != $this.id;
DEFINE FIELD chain      ON organization TYPE array<record<organization>>  DEFAULT [$this.id];
DEFINE INDEX idx_parent ON organization FIELDS parent;
DEFINE INDEX idx_chain  ON organization FIELDS chain;

DEFINE EVENT audit      ON organization WHEN $before != $after THEN {
    fn::audit_logger($before, $after);
};

-- If this event will be a bottle-neck, refactor it into lazy shadow task
DEFINE EVENT hierarchy_sync ON organization WHEN $before.parent != $after.parent THEN {
    LET $new_chain = IF $after.parent == NONE {
        [ $after.id ]
    } ELSE {
        LET $p_chain = (SELECT VALUE chain FROM $after.parent)[0];
        IF $after.id IN $p_chain {
            THROW "Circular reference detected";
        };
        array::add($p_chain, $after.id)
    };

    UPDATE $after.id SET chain = $new_chain;

    fn::sync_descendants_chain($after.id, $new_chain);
};

-- [DML: fn:sync_descendants_chain]
DEFINE FUNCTION fn::sync_descendants_chain($parent_id: record<organization>, $parent_chain: array) {
    LET $children = SELECT id FROM organization WHERE parent = $parent_id;

    FOR $child IN $children {
        LET $child_new_chain = array::add($parent_chain, $child.id);
        UPDATE $child.id SET chain = $child_new_chain;

        fn::sync_descendants_chain($child.id, $child_new_chain);
    };
};