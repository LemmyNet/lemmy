create table captcha_answer (
    id serial primary key,
    uuid uuid not null unique default gen_random_uuid(),
    answer text not null,
    published timestamp not null default now()
);
