create table captcha_answer (
    uuid text not null primary key,
    answer text not null,
    expires timestamp not null
);
