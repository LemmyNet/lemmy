-- These actually increased query time, but they prevent more postgres workers from being launched, and so should free up locks.
CREATE INDEX idx_post_published ON post (published);

CREATE INDEX idx_post_like_published ON post_like (published);

CREATE INDEX idx_comment_like_published ON comment_like (published);

