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
    and version like '$4%'
ORDER BY
    timestamp DESC,
    CAST(SUBSTR(version, 1, INSTR(version, '.') - 1) AS INTEGER) DESC,
    CAST(SUBSTR(version, INSTR(version, '.') + 1, INSTR(SUBSTR(version, INSTR(version, '.') + 1), '.') - 1) AS INTEGER) DESC,
    CAST(SUBSTR(version, INSTR(SUBSTR(version, INSTR(version, '.') + 1), '.') + INSTR(version, '.') + 1, 
                CASE WHEN INSTR(SUBSTR(version, INSTR(SUBSTR(version, INSTR(version, '.') + 1), '.') + INSTR(version, '.') + 1), '-') > 0 
                      THEN INSTR(SUBSTR(version, INSTR(SUBSTR(version, INSTR(version, '.') + 1), '.') + INSTR(version, '.') + 1), '-') - 1 
                      ELSE LENGTH(SUBSTR(version, INSTR(SUBSTR(version, INSTR(version, '.') + 1), '.') + INSTR(version, '.') + 1)) 
                END) AS INTEGER) DESC,
    CASE
      WHEN INSTR(version, '-') > 0 THEN SUBSTR(version, INSTR(version, '-') + 1)
      ELSE ''
    END DESC;