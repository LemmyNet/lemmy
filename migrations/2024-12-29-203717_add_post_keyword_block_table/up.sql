CREATE TABLE local_user_keyword_block (
    local_user_id int REFERENCES local_user (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    keyword varchar(50) NOT NULL,
    PRIMARY KEY (local_user_id, keyword)
);

