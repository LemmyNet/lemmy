CREATE TABLE password_reset_request (
    id serial PRIMARY KEY,
    user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    token_encrypted text NOT NULL,
    published timestamp NOT NULL DEFAULT now()
);

