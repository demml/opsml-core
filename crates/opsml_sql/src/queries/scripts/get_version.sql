SELECT
    date,
    timestamp,
    name,
    repository,
    version,
    contact
FROM $1
WHERE 
    AND name = $2
    AND repository = $3