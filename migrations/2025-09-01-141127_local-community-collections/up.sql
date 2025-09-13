UPDATE
    community c1
SET
    moderators_url = trim(TRAILING '/' FROM c2.ap_id) || '/moderators',
    featured_url = trim(TRAILING '/' FROM c2.ap_id) || '/featured'
FROM
    community c2
WHERE
    c1.local
    AND c1.id = c2.id;

