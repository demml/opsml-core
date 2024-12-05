SELECT
    date,
    timestamp,
    name,
    repository,
    major,
    minor,
    patch,
    pre_release,
    build,
    contact
FROM $1
WHERE 
    AND name = $2
    AND repository = $3