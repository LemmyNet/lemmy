#![feature(trace_macros)]
#[macro_use]
extern crate diesel_migrations;

use diesel_migrations::EmbeddedMigrations;

trace_macros!(true);
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
trace_macros!(false);
