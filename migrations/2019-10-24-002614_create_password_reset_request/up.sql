create table password_reset_request (
  id serial primary key,
  user_id int references user_ on update cascade on delete cascade not null,
  token_encrypted text not null,
  published timestamp not null default now()
);
