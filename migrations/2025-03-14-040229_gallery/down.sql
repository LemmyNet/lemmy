UPDATE
    post
SET
    url = post_gallery.url,
    alt_text = post_gallery.alt_text,
    url_content_type = post_gallery.url_content_type
FROM
    post_gallery
WHERE
    post_gallery.post_id = post.id
    AND post_gallery.page = 0;

DROP TABLE post_gallery;

