UPDATE
    post
SET
    url = post_url.url,
    alt_text = post_url.alt_text,
    url_content_type = post_url.url_content_type
FROM
    post_url
WHERE
    post_url.post_id = post.id and post_url.page = 0;

DROP TABLE post_url;

