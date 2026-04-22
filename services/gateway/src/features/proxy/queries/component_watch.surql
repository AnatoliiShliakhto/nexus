LIVE SELECT
    id.id() as component, target, route, protected
FROM
    component
WHERE
    (status = 'ACTIVE' OR status = 'SYSTEM')
    AND target != NONE
    AND route != NONE;