#![cfg(test)]
#![expect(clippy::expect_used)]
use itertools::Itertools;
use lemmy_utils::settings::SETTINGS;
use pathfinding::matrix::Matrix;
use std::{
  borrow::Cow,
  process::{Command, Stdio},
};

// It's not possible to call `export_snapshot()` for each dump and run the dumps in parallel with
// the `--snapshot` flag. Don't waste your time!!!!

/// Returns almost all things currently in the database, represented as SQL statements that would
/// recreate them.
pub(crate) fn get_dump() -> String {
  let db_url = SETTINGS.get_database_url();
  let output = Command::new("pg_dump")
    .args([
      // Specify database URL
      "--dbname",
      &db_url,
      // Allow differences in row data and old fast tables
      "--schema-only",
      "--exclude-table=comment_aggregates_fast",
      "--exclude-table=community_aggregates_fast",
      "--exclude-table=post_aggregates_fast",
      "--exclude-table=user_fast",
      // Ignore some things to reduce the amount of queries done by pg_dump
      "--no-owner",
      "--no-privileges",
      "--no-comments",
      "--no-publications",
      "--no-security-labels",
      "--no-subscriptions",
      "--no-table-access-method",
      "--no-tablespaces",
      "--no-large-objects",
    ])
    .stderr(Stdio::inherit())
    .output()
    .expect("failed to start pg_dump process");

  // TODO: use exit_ok method when it's stable
  assert!(output.status.success());

  String::from_utf8(output.stdout).expect("pg_dump output is not valid UTF-8 text")
}

/// Checks dumps returned by [`get_dump`] and panics if they differ in a way that indicates a
/// mistake in whatever was run in between the dumps.
///
/// The panic message shows `label_of_change_from_0_to_1` and a diff from `dumps[0]` to `dumps[1]`.
/// For example, if something only exists in `dumps[1]`, then the diff represents the addition of
/// that thing.
///
/// `label_of_change_from_0_to_1` must say something about the change from `dumps[0]` to `dumps[1]`,
/// not `dumps[1]` to `dumps[0]`. This requires the two `dumps` elements being in an order that fits
/// with `label_of_change_from_0_to_1`. This does not necessarily match the order in which the dumps
/// were created.
pub(crate) fn check_dump_diff(dumps: [&str; 2], label_of_change_from_0_to_1: &str) {
  let [sorted_statements_in_0, sorted_statements_in_1] = dumps.map(|dump| {
    dump
      .split("\n\n")
      .map(str::trim_start)
      .filter(|&chunk| !(is_ignored_trigger(chunk) || is_view(chunk) || is_comment(chunk)))
      .map(remove_ignored_uniqueness_from_statement)
      .sorted_unstable()
      .collect::<Vec<_>>()
  });
  let mut statements_only_in_0 = Vec::new();
  let mut statements_only_in_1 = Vec::new();
  for diff in diff::slice(&sorted_statements_in_0, &sorted_statements_in_1) {
    match diff {
      diff::Result::Left(statement) => statements_only_in_0.push(&**statement),
      diff::Result::Right(statement) => statements_only_in_1.push(&**statement),
      diff::Result::Both(_, _) => {}
    }
  }

  if !(statements_only_in_0.is_empty() && statements_only_in_1.is_empty()) {
    let (a, b): (String, String) = select_pairs([&statements_only_in_0, &statements_only_in_1])
      .flat_map(|[a, b]| [(a, b), ("\n\n", "\n\n")])
      .unzip();
    let diff = unified_diff::diff(a.as_bytes(), "", b.as_bytes(), "", 10000);
    panic!(
      "{label_of_change_from_0_to_1}\n\n{}",
      String::from_utf8_lossy(&diff)
    );
  }
}

fn is_ignored_trigger(chunk: &str) -> bool {
  [
    "refresh_comment_like",
    "refresh_comment",
    "refresh_community_follower",
    "refresh_community_user_ban",
    "refresh_community",
    "refresh_post_like",
    "refresh_post",
    "refresh_private_message",
    "refresh_user",
  ]
  .into_iter()
  .any(|trigger_name| {
    [("CREATE FUNCTION public.", '('), ("CREATE TRIGGER ", ' ')]
      .into_iter()
      .any(|(before, after)| {
        chunk
          .strip_prefix(before)
          .and_then(|s| s.strip_prefix(trigger_name))
          .is_some_and(|s| s.starts_with(after))
      })
  })
}

fn is_view(chunk: &str) -> bool {
  [
    "CREATE VIEW ",
    "CREATE OR REPLACE VIEW ",
    "CREATE MATERIALIZED VIEW ",
  ]
  .into_iter()
  .any(|prefix| chunk.starts_with(prefix))
}

fn is_comment(s: &str) -> bool {
  s.lines().all(|line| line.starts_with("--"))
}

fn remove_ignored_uniqueness_from_statement(statement: &str) -> Cow<'_, str> {
  // Sort column names, so differences in column order are ignored
  if statement.starts_with("CREATE TABLE ") {
    let mut lines = statement
      .lines()
      .map(|line| line.strip_suffix(',').unwrap_or(line))
      .collect::<Vec<_>>();

    sort_within_sections(&mut lines, |line| {
      match line.chars().next() {
        // CREATE
        Some('C') => 0,
        // Indented column name
        Some(' ') => 1,
        // End of column list
        Some(')') => 2,
        _ => panic!("unrecognized part of `CREATE TABLE` statement: {line}"),
      }
    });

    Cow::Owned(lines.join("\n"))
  } else {
    Cow::Borrowed(statement)
  }
}

fn sort_within_sections<T: Ord + ?Sized>(vec: &mut [&T], mut section: impl FnMut(&T) -> u8) {
  vec.sort_unstable_by_key(|&i| (section(i), i));
}

/// For each string in list 0, makes a guess of which string in list 1 is a variant of it (or vice
/// versa).
fn select_pairs<'a>([a, b]: [&'a [&'a str]; 2]) -> impl Iterator<Item = [&'a str; 2]> {
  let len = std::cmp::max(a.len(), b.len());
  let get_candidate_pair_at =
    |(row, column)| [a.get(row), b.get(column)].map(|item| *item.unwrap_or(&""));
  let difference_amounts = Matrix::from_fn(len, len, |position| {
    amount_of_difference_between(get_candidate_pair_at(position))
  });
  pathfinding::kuhn_munkres::kuhn_munkres_min(&difference_amounts)
    .1
    .into_iter()
    .enumerate()
    .map(get_candidate_pair_at)
}

/// Computes string distance, using the already required [`diff`] crate to avoid adding another
/// dependency.
fn amount_of_difference_between([a, b]: [&str; 2]) -> isize {
  diff::chars(a, b)
    .into_iter()
    .filter(|i| !matches!(i, diff::Result::Both(_, _)))
    .fold(0, |count, _| count.saturating_add(1))
}

/// Makes sure the after dump does not contain any DEFERRABLE constraints.
pub(crate) fn deferr_constraint_check(dump: &str) {
  if dump.contains(" DEFERR") {
    panic!("Schema should not have DEFER constraints.")
  }
}

// `#[cfg(test)]` would be redundant here
mod tests {
  #[test]
  fn test_select_pairs() {
    let x = "Cupcake";
    let x_variant = "Cupcaaaaake";
    let y = "eee";
    let y_variant = "ee";
    let z = "bruh";
    assert_eq!(
      super::select_pairs([&[x, y, z], &[y_variant, x_variant]]).collect::<Vec<_>>(),
      vec![[x, x_variant], [y, y_variant], [z, ""]]
    );
  }
}
