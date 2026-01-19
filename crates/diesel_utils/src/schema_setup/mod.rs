mod diff_check;
use anyhow::Context;
use diesel::{
  BoolExpressionMethods,
  Connection,
  ExpressionMethods,
  PgConnection,
  QueryDsl,
  RunQueryDsl,
  connection::SimpleConnection,
  dsl::exists,
  migration::{Migration, MigrationVersion},
  pg::Pg,
  select,
  update,
};
use diesel_migrations::MigrationHarness;

// `?` can't convert `diesel::migration::Result` to some other types because of https://github.com/dtolnay/anyhow/issues/66

diesel::table! {
  pg_namespace (nspname) {
    nspname -> Text,
  }
}

diesel::table! {
  previously_run_sql (id) {
    id -> Bool,
    content -> Text,
  }
}

pub const MIGRATIONS: diesel_migrations::EmbeddedMigrations =
  diesel_migrations::embed_migrations!();

/// This SQL code sets up the `r` schema, which contains things that can be safely dropped and
/// replaced instead of being changed using migrations. It may not create or modify things outside
/// of the `r` schema (indicated by `r.` before the name), unless a comment says otherwise.
fn replaceable_schema() -> String {
  [
    "CREATE SCHEMA r;",
    include_str!("../../replaceable_schema/utils.sql"),
    include_str!("../../replaceable_schema/triggers.sql"),
  ]
  .join("\n")
}

const REPLACEABLE_SCHEMA_PATH: &str = "crates/diesel_utils/replaceable_schema";

pub struct MigrationHarnessWrapper {
  // Migrations don't support async connection, and non-async migration execution is okay
  pub conn: PgConnection,
}

impl MigrationHarness<Pg> for MigrationHarnessWrapper {
  fn run_migration(
    &mut self,
    migration: &dyn Migration<Pg>,
  ) -> diesel::migration::Result<MigrationVersion<'static>> {
    // Drop `r` schema, so migrations don't need to be made to work both with and without things in
    // it existing
    self.revert_replaceable_schema()?;

    self.conn.run_migration(migration)
  }

  fn revert_migration(
    &mut self,
    migration: &dyn Migration<Pg>,
  ) -> diesel::migration::Result<MigrationVersion<'static>> {
    // Drop `r` schema, so migrations don't need to be made to work both with and without things in
    // it existing
    self.revert_replaceable_schema()?;

    self.conn.revert_migration(migration)
  }

  fn applied_migrations(&mut self) -> diesel::migration::Result<Vec<MigrationVersion<'static>>> {
    self.conn.applied_migrations()
  }
}

impl MigrationHarnessWrapper {
  pub fn new(db_url: &str) -> anyhow::Result<Self> {
    Ok(MigrationHarnessWrapper {
      conn: PgConnection::establish(db_url)?,
    })
  }

  pub fn need_schema_setup(&mut self) -> anyhow::Result<bool> {
    Ok(
      self
        .conn
        .has_pending_migration(MIGRATIONS)
        .map_err(anyhow::Error::from_boxed)?
        || !self.replaceable_schema_is_up_to_date()?,
    )
  }

  fn replaceable_schema_is_up_to_date(&mut self) -> anyhow::Result<bool> {
    // Assumes that the migration that creates the previously_run_sql table was already run. This
    // assumption is true if has_pending_migration already returned false.
    let sql_unchanged = exists(
      previously_run_sql::table.filter(previously_run_sql::content.eq(replaceable_schema())),
    );

    let schema_exists = exists(pg_namespace::table.find("r"));

    Ok(select(sql_unchanged.and(schema_exists)).get_result(&mut self.conn)?)
  }

  /// this shouldn't be run in the same transaction as the next stuff, since [todo finish
  /// explanation]
  fn revert_replaceable_schema(&mut self) -> anyhow::Result<()> {
    self
      .conn
      .batch_execute("DROP SCHEMA IF EXISTS r CASCADE;")
      .with_context(|| format!("Failed to revert SQL files in {REPLACEABLE_SCHEMA_PATH}"))?;

    // Value in `previously_run_sql` table is not set here because the table might not exist,
    // and that's fine because the existence of the `r` schema is also checked

    Ok(())
  }

  pub fn run_replaceable_schema(&mut self) -> anyhow::Result<()> {
    self.revert_replaceable_schema()?;

    self.conn.transaction(|conn| {
      conn
        .batch_execute(&replaceable_schema())
        .with_context(|| format!("Failed to run SQL files in {REPLACEABLE_SCHEMA_PATH}"))?;

      let num_rows_updated = update(previously_run_sql::table)
        .set(previously_run_sql::content.eq(replaceable_schema()))
        .execute(conn)?;

      debug_assert_eq!(num_rows_updated, 1);

      Ok(())
    })
  }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::unwrap_used)]
mod tests {
  use super::*;
  use anyhow::anyhow;
  use diesel::{
    dsl::{not, sql},
    sql_types,
  };
  use diesel_ltree::Ltree;
  use lemmy_utils::{error::LemmyErrorExt2, settings::SETTINGS};
  use serial_test::serial;
  // The number of migrations that should be run to set up some test data.
  // Currently, this includes migrations until
  // 2020-04-07-135912_add_user_community_apub_constraints, since there are some mandatory apub
  // fields need to be added.

  const INITIAL_MIGRATIONS_COUNT: u64 = 40;

  // Test data IDs
  const TEST_USER_ID_1: i32 = 101;
  const USER1_NAME: &str = "test_user_1";
  const USER1_ACTOR_ID: &str = "test_user_1@fedi.example";
  const USER1_PREFERRED_NAME: &str = "preferred_1";
  const USER1_EMAIL: &str = "email1@example.com";
  const USER1_PASSWORD: &str = "test_password_1";
  const USER1_PUBLIC_KEY: &str = "test_public_key_1";

  const TEST_USER_ID_2: i32 = 102;
  const USER2_NAME: &str = "test_user_2";
  const USER2_ACTOR_ID: &str = "test_user_2@fedi.example";
  const USER2_PREFERRED_NAME: &str = "preferred2";
  const USER2_EMAIL: &str = "email2@example.com";
  const USER2_PASSWORD: &str = "test_password_2";
  const USER2_PUBLIC_KEY: &str = "test_public_key_2";

  const TEST_COMMUNITY_ID_1: i32 = 101;
  const COMMUNITY_NAME: &str = "test_community_1";
  const COMMUNITY_TITLE: &str = "Test Community 1";
  const COMMUNITY_DESCRIPTION: &str = "This is a test community.";
  const CATEGORY_ID: i32 = 4; // Should be a valid category "Movies"
  const COMMUNITY_ACTOR_ID: &str = "https://fedi.example/community/12345";
  const COMMUNITY_PUBLIC_KEY: &str = "test_public_key_community_1";

  const TEST_POST_ID_1: i32 = 101;
  const POST_NAME: &str = "Post Title";
  const POST_URL: &str = "https://fedi.example/post/12345";
  const POST_BODY: &str = "Post Body.";
  const POST_AP_ID: &str = "https://fedi.example/post/12345";

  const TEST_COMMENT_ID_1: i32 = 101;
  const COMMENT1_CONTENT: &str = "Comment";
  const COMMENT1_AP_ID: &str = "https://fedi.example/comment/12345";

  const TEST_COMMENT_ID_2: i32 = 102;
  const COMMENT2_CONTENT: &str = "Reply";
  const COMMENT2_AP_ID: &str = "https://fedi.example/comment/12346";

  #[test]
  #[serial]
  // todo: maybe add commends for need_schema_setup asserts
  fn test_schema_setup() -> diesel::migration::Result<()> {
    let db_url = SETTINGS.get_database_url_with_options().into_anyhow()?;
    let mut harness = crate::schema_setup::MigrationHarnessWrapper::new(&db_url)?;

    // Start with consistent state by dropping everything
    harness.conn.batch_execute("DROP OWNED BY CURRENT_USER;")?;

    assert!(harness.need_schema_setup()?);

    // Run initial migrations to prepare basic tables
    harness.run_pending_migrations_in_range(
      MIGRATIONS,
      diesel_migrations::Range::NumberOfMigrations(INITIAL_MIGRATIONS_COUNT),
    )?;

    assert!(harness.need_schema_setup()?);

    // Insert the test data
    insert_test_data(&mut harness.conn)?;

    // Run all migrations, and make sure that changes can be correctly reverted
    for migration in harness.pending_migrations(MIGRATIONS)? {
      let before = diff_check::get_dump();

      harness.run_migration(&migration)?;
      harness.revert_migration(&migration)?;

      let after = diff_check::get_dump();

      diff_check::check_dump_diff(
        [&after, &before],
        &format!(
          "These changes need to be applied in migrations/{}/down.sql:",
          migration.name()
        ),
      );

      harness.run_migration(&migration)?;
    }

    assert!(harness.need_schema_setup()?);

    // Make sure that replaceable schema can be correctly reverted
    let before = diff_check::get_dump();

    harness.run_replaceable_schema()?;
    harness.revert_replaceable_schema()?;

    let after = diff_check::get_dump();

    diff_check::check_dump_diff(
      [&before, &after],
      "The code in crates/diesel_utils/replaceable_schema incorrectly created or modified things outside of the `r` schema, causing these changes to be left behind after dropping the schema:",
    );

    assert!(harness.need_schema_setup()?);
    harness.run_replaceable_schema()?;
    assert!(!harness.need_schema_setup()?);

    // Check the test data we inserted before after running migrations
    check_test_data(&mut harness.conn)?;

    // Check the current schema
    assert_eq!(
      get_foreign_keys_with_missing_indexes(&mut harness.conn)?,
      Vec::<String>::new(),
      "each foreign key needs an index so that deleting the referenced row does not scan the whole referencing table"
    );
    diff_check::deferr_constraint_check(&after);

    // Todo: maybe clean up (this used to be for testing the limit option)
    harness.revert_last_migration(MIGRATIONS)?;
    assert!(harness.need_schema_setup()?);
    harness.run_next_migration(MIGRATIONS)?;
    harness.run_replaceable_schema()?;
    assert!(!harness.need_schema_setup()?);

    // Get a new connection, workaround for error `cache lookup failed for function 26633`
    // on `migrations/2025-10-15-114811-0000_merge-modlog-tables/down.sql`.
    harness.conn = PgConnection::establish(&db_url)?;

    // This should throw an error saying to use lemmy_server instead of diesel CLI, since
    // application_name isn't set to lemmy
    harness
      .conn
      .batch_execute("DROP OWNED BY CURRENT_USER; SET application_name=reddit;")?;
    assert!(matches!(
      harness.run_pending_migrations(MIGRATIONS),
      Err(e) if e.to_string().contains("lemmy_server")
    ));

    // Diesel CLI's way of running migrations shouldn't break the custom migration runner
    harness.conn.batch_execute("SET application_name=lemmy;")?;
    harness.run_pending_migrations(MIGRATIONS)?;
    harness.run_replaceable_schema()?;
    assert!(!harness.need_schema_setup()?);

    Ok(())
  }

  fn insert_test_data(conn: &mut PgConnection) -> anyhow::Result<()> {
    // Users
    conn.batch_execute(&format!(
      "INSERT INTO user_ (id, name, actor_id, preferred_username, password_encrypted, email, public_key) \
          VALUES ({}, '{}', '{}', '{}', '{}', '{}', '{}')",
      TEST_USER_ID_1,
      USER1_NAME,
      USER1_ACTOR_ID,
      USER1_PREFERRED_NAME,
      USER1_PASSWORD,
      USER1_EMAIL,
      USER1_PUBLIC_KEY
    ))?;

    conn.batch_execute(&format!(
      "INSERT INTO user_ (id, name, actor_id, preferred_username, password_encrypted, email, public_key) \
          VALUES ({}, '{}', '{}', '{}', '{}', '{}', '{}')",
      TEST_USER_ID_2,
      USER2_NAME,
      USER2_ACTOR_ID,
      USER2_PREFERRED_NAME,
      USER2_PASSWORD,
      USER2_EMAIL,
      USER2_PUBLIC_KEY
    ))?;

    // Community
    conn.batch_execute(&format!(
      "INSERT INTO community (id, actor_id, public_key, name, title, description, category_id, creator_id) \
          VALUES ({}, '{}', '{}', '{}', '{}', '{}', {}, {})",
      TEST_COMMUNITY_ID_1,
      COMMUNITY_ACTOR_ID,
      COMMUNITY_PUBLIC_KEY,
      COMMUNITY_NAME,
      COMMUNITY_TITLE,
      COMMUNITY_DESCRIPTION,
      CATEGORY_ID,
      TEST_USER_ID_1
    ))?;

    conn.batch_execute(&format!(
      "INSERT INTO community_moderator (community_id, user_id) \
          VALUES ({}, {})",
      TEST_COMMUNITY_ID_1, TEST_USER_ID_1
    ))?;

    // Post
    conn.batch_execute(&format!(
      "INSERT INTO post (id, name, url, body, creator_id, community_id, ap_id) \
          VALUES ({}, '{}', '{}', '{}', {}, {}, '{}')",
      TEST_POST_ID_1,
      POST_NAME,
      POST_URL,
      POST_BODY,
      TEST_USER_ID_1,
      TEST_COMMUNITY_ID_1,
      POST_AP_ID
    ))?;

    // Comment
    conn.batch_execute(&format!(
      "INSERT INTO comment (id, creator_id, post_id, parent_id, content, ap_id) \
           VALUES ({}, {}, {}, NULL, '{}', '{}')",
      TEST_COMMENT_ID_1, TEST_USER_ID_2, TEST_POST_ID_1, COMMENT1_CONTENT, COMMENT1_AP_ID
    ))?;

    conn.batch_execute(&format!(
      "INSERT INTO comment (id, creator_id, post_id, parent_id, content, ap_id) \
           VALUES ({}, {}, {}, {}, '{}', '{}')",
      TEST_COMMENT_ID_2,
      TEST_USER_ID_1,
      TEST_POST_ID_1,
      TEST_COMMENT_ID_1,
      COMMENT2_CONTENT,
      COMMENT2_AP_ID
    ))?;

    conn.batch_execute(&format!(
      "INSERT INTO comment_like (user_id, comment_id, post_id, score) \
           VALUES ({}, {}, {}, {})",
      TEST_USER_ID_1, TEST_COMMENT_ID_1, TEST_POST_ID_1, 1
    ))?;

    Ok(())
  }

  fn check_test_data(conn: &mut PgConnection) -> anyhow::Result<()> {
    use lemmy_db_schema_file::schema::{comment, community, notification, person, post};

    // Check users
    let users: Vec<(i32, String, Option<String>, String, String)> = person::table
      .select((
        person::id,
        person::name,
        person::display_name,
        person::ap_id,
        person::public_key,
      ))
      .order_by(person::id)
      .load(conn)
      .map_err(|e| anyhow!("Failed to read users: {}", e))?;

    assert_eq!(users.len(), 2);
    assert_eq!(users[0].0, TEST_USER_ID_1);
    assert_eq!(users[0].1, USER1_NAME);
    assert_eq!(users[0].2.clone().unwrap(), USER1_PREFERRED_NAME);
    assert_eq!(users[0].3, USER1_ACTOR_ID);
    assert_eq!(users[0].4, USER1_PUBLIC_KEY);

    assert_eq!(users[1].0, TEST_USER_ID_2);
    assert_eq!(users[1].1, USER2_NAME);
    assert_eq!(users[1].2.clone().unwrap(), USER2_PREFERRED_NAME);
    assert_eq!(users[1].3, USER2_ACTOR_ID);
    assert_eq!(users[1].4, USER2_PUBLIC_KEY);

    // Check communities
    let communities: Vec<(i32, String, String, String)> = community::table
      .select((
        community::id,
        community::name,
        community::ap_id,
        community::public_key,
      ))
      .load(conn)
      .map_err(|e| anyhow!("Failed to read communities: {}", e))?;

    assert_eq!(communities.len(), 1);
    assert_eq!(communities[0].0, TEST_COMMUNITY_ID_1);
    assert_eq!(communities[0].1, COMMUNITY_NAME);
    assert_eq!(communities[0].2, COMMUNITY_ACTOR_ID);
    assert_eq!(communities[0].3, COMMUNITY_PUBLIC_KEY);

    let posts: Vec<(i32, String, String, Option<String>, i32, i32)> = post::table
      .select((
        post::id,
        post::name,
        post::ap_id,
        post::body,
        post::community_id,
        post::creator_id,
      ))
      .load(conn)
      .map_err(|e| anyhow!("Failed to read posts: {}", e))?;

    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].0, TEST_POST_ID_1);
    assert_eq!(posts[0].1, POST_NAME);
    assert_eq!(posts[0].2, POST_AP_ID);
    assert_eq!(posts[0].3.clone().unwrap(), POST_BODY);
    assert_eq!(posts[0].4, TEST_COMMUNITY_ID_1);
    assert_eq!(posts[0].5, TEST_USER_ID_1);

    let comments: Vec<(i32, String, String, i32, i32, Ltree, i32)> = comment::table
      .select((
        comment::id,
        comment::content,
        comment::ap_id,
        comment::post_id,
        comment::creator_id,
        comment::path,
        comment::upvotes,
      ))
      .order_by(comment::id)
      .load(conn)
      .map_err(|e| anyhow!("Failed to read comments: {}", e))?;

    assert_eq!(comments.len(), 2);
    assert_eq!(comments[0].0, TEST_COMMENT_ID_1);
    assert_eq!(comments[0].1, COMMENT1_CONTENT);
    assert_eq!(comments[0].2, COMMENT1_AP_ID);
    assert_eq!(comments[0].3, TEST_POST_ID_1);
    assert_eq!(comments[0].4, TEST_USER_ID_2);
    assert_eq!(
      comments[0].5,
      Ltree(format!("0.{}", TEST_COMMENT_ID_1).to_string())
    );
    assert_eq!(comments[0].6, 1); // One upvote

    assert_eq!(comments[1].0, TEST_COMMENT_ID_2);
    assert_eq!(comments[1].1, COMMENT2_CONTENT);
    assert_eq!(comments[1].2, COMMENT2_AP_ID);
    assert_eq!(comments[1].3, TEST_POST_ID_1);
    assert_eq!(comments[1].4, TEST_USER_ID_1);
    assert_eq!(
      comments[1].5,
      Ltree(format!("0.{}.{}", TEST_COMMENT_ID_1, TEST_COMMENT_ID_2).to_string())
    );
    assert_eq!(comments[1].6, 0); // Zero upvotes

    // Check comment replies
    let replies: Vec<(Option<i32>, i32)> = notification::table
      .select((notification::comment_id, notification::recipient_id))
      .order_by(notification::comment_id)
      .load(conn)
      .map_err(|e| anyhow!("Failed to read comment replies: {}", e))?;

    assert_eq!(replies.len(), 2);
    assert_eq!(replies[0].0, Some(TEST_COMMENT_ID_1));
    assert_eq!(replies[0].1, TEST_USER_ID_1);
    assert_eq!(replies[1].0, Some(TEST_COMMENT_ID_2));
    assert_eq!(replies[1].1, TEST_USER_ID_2);

    Ok(())
  }

  const FOREIGN_KEY: &str = "f";

  fn get_foreign_keys_with_missing_indexes(conn: &mut PgConnection) -> anyhow::Result<Vec<String>> {
    diesel::table! {
      pg_constraint (table_oid, name, kind, column_numbers) {
        #[sql_name = "conrelid"]
        table_oid -> Oid,
        #[sql_name = "conname"]
        name -> Text,
        #[sql_name = "contype"]
        kind -> Text,
        #[sql_name = "conkey"]
        column_numbers -> Array<Int2>,
      }
    }

    diesel::table! {
      pg_index (table_oid, key_length, column_numbers) {
        #[sql_name = "indrelid"]
        table_oid -> Oid,
        #[sql_name = "indnkeyatts"]
        key_length -> Int2,
        #[sql_name = "indkey"]
        column_numbers -> Array<Int2>,
      }
    }

    diesel::allow_tables_to_appear_in_same_query!(pg_constraint, pg_index);

    let matching_index = pg_index::table
      .filter(pg_index::table_oid.eq(pg_constraint::table_oid))
      // Check if the index's key (not columns listed with `INCLUDE`) starts with the foreign key.
      // TODO: use Diesel array slice function when it's added.
      .filter(sql::<sql_types::Bool>(
        "((pg_index.indkey[:pg_index.indnkeyatts])[:array_length(pg_constraint.conkey, 1)] = pg_constraint.conkey)"
      ));

    let res = pg_constraint::table
      .select(pg_constraint::name)
      .filter(pg_constraint::kind.eq(FOREIGN_KEY))
      .filter(not(exists(matching_index)))
      .load(conn)?;

    Ok(res)
  }
}
