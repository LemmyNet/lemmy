#![cfg(test)]
#![expect(clippy::expect_used)]
use itertools::Itertools;
use lemmy_utils::settings::SETTINGS;
use std::{
  borrow::Cow,
  collections::{btree_set::BTreeSet, HashSet},
  process::{Command, Stdio},
};

// It's not possible to call `export_snapshot()` for each dump and run the dumps in parallel with
// the `--snapshot` flag. Don't waste your time!!!!

pub fn get_dump() -> String {
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

pub fn check_dump_diff(mut dumps: [&str; 2], label_of_change_from_dump_0_to_dump_1: &str) {
  // Performance optimizations
  if dumps[0] == dumps[1] {
    return;
  }
  dumps = trim_matching_chunks_at_beginning_and_end(dumps);

  let [before_chunks, after_chunks] = dumps.map(|dump| {
    dump
      .split("\n\n")
      .filter_map(remove_ignored_details_from_chunk)
      // Sort
      .collect::<BTreeSet<_>>()
      .into_iter()
      .collect::<Vec<_>>()
  });
  let diff_results = diff::slice(&before_chunks, &after_chunks);
  let mut only_in_before = HashSet::new();
  let mut only_in_after = HashSet::new();
  for res in diff_results {
    match res {
      diff::Result::Left(chunk) => only_in_before.insert(&**chunk),
      diff::Result::Right(chunk) => only_in_after.insert(&**chunk),
      diff::Result::Both(_, _) => continue,
    };
  }

  if only_in_before.is_empty() && only_in_after.is_empty() {
    return;
  }

  // Build the panic message

  // All possible pairs of an item in only_in_before and an item in only_in_after
  let mut maybe_in_both = only_in_before
    .iter()
    .flat_map(|&a| {
      only_in_after
        .iter()
        .map(move |&b| (chunk_difference_amount(a, b), a, b))
    })
    .collect::<BTreeSet<_>>();

  // Determine which item in only_in_before corresponds with which item in only_in_after, kinda like
  // what git does to detect which old file corresponds to which new file if it was renamed
  #[expect(clippy::needless_collect)]
  let in_both = std::iter::from_fn(|| {
    // Get the pair with minimum difference amount
    let (_, item_in_before, item_in_after) = maybe_in_both.pop_first()?;

    // Remove alternative pairings of these chunks
    maybe_in_both.retain(|&(_, other_in_before, other_in_after)| {
      other_in_before != item_in_before && other_in_after != item_in_after
    });

    // Remove these chunks from only_in_before and only_in_after
    only_in_before.remove(item_in_before);
    only_in_after.remove(item_in_after);

    Some((item_in_before, item_in_after))
  })
  // Finish all changes to only_in_before and only_in_after before using the iterators
  .collect::<Vec<_>>();

  let header = format!("{label_of_change_from_dump_0_to_dump_1}\n\n");

  let diffs = in_both
    .into_iter()
    .chain(only_in_before.into_iter().map(|i| (i, "")))
    .chain(only_in_after.into_iter().map(|i| ("", i)))
    .flat_map(|(before, after)| {
      diff::lines(before, after)
        .into_iter()
        .flat_map(|line| match line {
          diff::Result::Left(s) => ["- ", s, "\n"],
          diff::Result::Right(s) => ["+ ", s, "\n"],
          diff::Result::Both(s, _) => ["  ", s, "\n"],
        })
        .chain(["\n"]) // Blank line after each chunk diff
    });

  panic!(
    "{}",
    std::iter::once(header.as_str())
      .chain(diffs)
      .collect::<String>()
  );
}

fn trim_matching_chunks_at_beginning_and_end(dumps: [&str; 2]) -> [&str; 2] {
  let len_of_match_at_beginning =
    count_bytes_until_chunks_dont_match(dumps.map(|dump| dump.as_bytes().iter()));
  let len_of_match_at_end =
    count_bytes_until_chunks_dont_match(dumps.map(|dump| dump.as_bytes().iter().rev()));
  dumps.map(|dump| {
    dump
      .get(len_of_match_at_beginning..(dump.len() - len_of_match_at_end))
      .expect("invalid count_bytes_until_chunks_dont_match result")
  })
}

fn count_bytes_until_chunks_dont_match<'a>(
  [iter0, iter1]: [impl DoubleEndedIterator<Item = &'a u8> + ExactSizeIterator + Clone; 2],
) -> usize {
  // iter0: FOO\n\nBAR\n\nBUNNY
  // iter1: FOO\n\nBAR\n\nBURROW
  //        ^^^^^^^^^^^^^^^^ matching_len
  //                      ^^ partial_match_len
  //        ^^^^^^^^^^^^^^   the returned number
  let matching_len = iter0
    .clone()
    .zip(iter1.clone())
    .take_while(|(a, b)| a == b)
    .count();
  let partial_match_len = if [&iter0, &iter1]
    .into_iter()
    .all(|i| match_end_aligns_with_chunk_end(i.clone(), matching_len))
  {
    0
  } else {
    iter0
      .take(matching_len)
      .rev()
      .copied()
      .tuple_windows::<(_, _)>()
      .take_while(|&a| a != (b'\n', b'\n'))
      .count()
  };
  matching_len - partial_match_len
}

fn match_end_aligns_with_chunk_end<'a>(
  iter: impl DoubleEndedIterator<Item = &'a u8> + ExactSizeIterator + Clone,
  matching_len: usize,
) -> bool {
  iter
    .skip(matching_len)
    .chain(b"\n\n") // treat end of string as a chunk boundary
    .take(2)
    .eq(b"\n\n")
}

fn chunk_difference_amount(a: &str, b: &str) -> Vec<usize> {
  // Prioritize similarity of specific parts by returning the difference amount in each
  // part, starting with higher priority
  chunk_parts_for_pair_selection(a)
    .into_iter()
    .zip(chunk_parts_for_pair_selection(b))
    .map(|(a, b)| count_inserted_or_deleted_chars_in_diff(a, b))
    .collect::<Vec<_>>()
}

fn chunk_parts_for_pair_selection(chunk: &str) -> [&str; 3] {
  let (command_name, after_command_name) = chunk
    .split_once(|c: char| c.is_lowercase())
    .unwrap_or(("", chunk));
  // First line typically includes the name of a table, function, etc.
  let (remainder_of_first_line, after_first_line) = after_command_name
    .split_once('\n')
    .unwrap_or(("", after_command_name));
  [command_name, remainder_of_first_line, after_first_line]
}

fn count_inserted_or_deleted_chars_in_diff(a: &str, b: &str) -> usize {
  diff::chars(a, b)
    .into_iter()
    .filter(|i| !matches!(i, diff::Result::Both(_, _)))
    .count()
}

fn remove_ignored_details_from_chunk(mut chunk: &str) -> Option<Cow<'_, str>> {
  while let Some(s) = trim_start_of_chunk(chunk) {
    chunk = s;
  }
  if chunk.is_empty() ||
  // Skip old views and fast table triggers
  chunk.strip_prefix("CREATE ").is_some_and(|c| {
    c
      .starts_with("VIEW ")
      || c.starts_with("OR REPLACE VIEW ")
      || c.starts_with("MATERIALIZED VIEW ")
      || c.strip_prefix("FUNCTION public.")
          .and_then(after_skipped_trigger_name)
          .is_some_and(|a| a.starts_with('('))
      ||
        c.strip_prefix("TRIGGER ")
          .and_then(after_skipped_trigger_name)
          .is_some_and(|a| a.starts_with(' '))
  }) {
    return None;
  }
  let mut chunk = Cow::Borrowed(chunk);

  // Sort column names, so differences in column order are ignored
  if chunk.starts_with("CREATE TABLE ") {
    let mut lines = chunk
      .lines()
      .map(|line| line.strip_suffix(',').unwrap_or(line))
      .collect::<Vec<_>>();

    sort_within_sections(&mut lines, |line| {
      match line.chars().next() {
        // CREATE
        Some('C') => 0,
        // Indented column name
        Some(' ') => 1,
        // End
        Some(')') => 2,
        _ => panic!("unrecognized part of `CREATE TABLE` statement: {line}"),
      }
    });

    chunk = Cow::Owned(lines.join("\n"));
  }

  Some(chunk)
}

fn sort_within_sections<T: Ord + ?Sized>(vec: &mut [&T], mut section: impl FnMut(&T) -> u8) {
  vec.sort_unstable_by_key(|&i| (section(i), i));
}

fn trim_start_of_chunk(s: &str) -> Option<&str> {
  if let Some(after) = s.strip_prefix("--") {
    // Skip commented line
    Some(after_first_occurence(after, "\n"))
  } else if let Some(after) = s.strip_prefix(char::is_whitespace) {
    // Skip whitespace
    Some(after.trim_start())
  } else {
    None
  }
}

fn after_first_occurence<'a>(s: &'a str, pat: &str) -> &'a str {
  s.split_once(pat).unwrap_or_default().1
}

fn after_skipped_trigger_name(s: &str) -> Option<&str> {
  s.strip_prefix("refresh_comment_like")
    .or_else(|| s.strip_prefix("refresh_comment"))
    .or_else(|| s.strip_prefix("refresh_community_follower"))
    .or_else(|| s.strip_prefix("refresh_community_user_ban"))
    .or_else(|| s.strip_prefix("refresh_community"))
    .or_else(|| s.strip_prefix("refresh_post_like"))
    .or_else(|| s.strip_prefix("refresh_post"))
    .or_else(|| s.strip_prefix("refresh_private_message"))
    .or_else(|| s.strip_prefix("refresh_user"))
}

// cfg(test) would be redundant here
mod tests {
  #[test]
  fn test_count_bytes_until_chunks_dont_match() {
    let a = b"FOO\n\nFOO\n\nFOO\nFOO";
    let b = b"FOO\n\nFOO\n\nFOO\nBAR";
    let c = b"FOO\n\nFOO\n\nBUNNY";
    let d = b"FOO\n\nFOO\n\n";
    assert_eq!(
      super::count_bytes_until_chunks_dont_match([a.iter(), b.iter()]),
      d.len()
    );
    assert_eq!(
      super::count_bytes_until_chunks_dont_match([a.iter(), c.iter()]),
      d.len()
    );
    assert_eq!(
      super::count_bytes_until_chunks_dont_match([a.iter(), a.iter()]),
      a.len()
    );
    assert_eq!(
      super::count_bytes_until_chunks_dont_match([b"z".iter(), b"z".iter()]),
      1
    );
    assert_eq!(
      super::count_bytes_until_chunks_dont_match([b"z".iter(), b"y".iter()]),
      0
    );
  }
}
