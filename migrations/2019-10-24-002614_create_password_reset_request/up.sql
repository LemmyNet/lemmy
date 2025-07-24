CREATE TABLE password_reset_request (
    id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    token_encrypted text NOT NULL,
    published timestamp NOT NULL DEFAULT now()
);

