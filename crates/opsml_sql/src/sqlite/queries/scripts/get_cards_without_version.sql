SELECT
    date,
    timestamp,
    name,
    repository,
    major,
    minor,
    patch,
    pre_tag,
    build_tag,
    contact
FROM $1
WHERE 
    AND name = $2
    AND repository = $3
    
ORDER BY
    timestamp DESC
LIMIT 20;