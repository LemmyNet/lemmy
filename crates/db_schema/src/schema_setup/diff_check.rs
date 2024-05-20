use diesel::{PgConnection, RunQueryDsl,connection::SimpleConnection,Connection};
use lemmy_utils::settings::SETTINGS;
use std::{
  borrow::Cow,
  collections::BTreeSet,
  fmt::Write,
  process::{Command, Stdio},thread,cell::OnceCell,sync::{Arc,Mutex},collections::HashMap,any::Any
};
use crossbeam_channel::{Sender, Receiver};

enum DumpAction {
  Send(Sender<String>),
  Compare(Receiver<String>, String),
}

pub struct DiffChecker {
  snapshot_conn: PgConnection,
  handles: Vec<thread::JoinHandle<()>>,
  snapshot_sender: Option<Sender<(String, DumpAction)>>,
  error: Receiver<Box<dyn Any + Send + 'static>>,
  // todo rename to channels
  //dump_receivers: Arc<Mutex<HashMap<String, Receiver<String>>>>,
}

diesel::sql_function! {
  fn pg_export_snapshot() -> diesel::sql_types::Text;
}

impl DiffChecker {
  pub fn new(db_url: &str) -> diesel::result::QueryResult<Self> {
    // todo use settings
    let mut snapshot_conn = PgConnection::establish(db_url).expect("conn");
    snapshot_conn.batch_execute("BEGIN;")?;
    let (tx, rx) = crossbeam_channel::unbounded();
    let (error_t, error_r) = crossbeam_channel::unbounded();
    //let dump_receivers = Arc::new(Mutex::new(HashMap::new()));

    let mut handles = Vec::new();
    let n = usize::from(thread::available_parallelism().expect("parallelism"));
    // todo remove
    assert_eq!(16,n);
    for _ in 0..(n){
      let rx2 = rx.clone();
      let error_t = error_t.clone();
      handles.push(thread::spawn(move || if let Err(e) = std::panic::catch_unwind(move || {
        while let Ok((snapshot, action)) = rx2.recv() {
          let snapshot_arg = format!("--snapshot={snapshot}");
          let output = Command::new("pg_dump")
            .args(["--schema-only", &snapshot_arg])
            .env("DATABASE_URL", SETTINGS.get_database_url())
            .output()
            .expect("failed to start pg_dump process");
          
          if !output.status.success() {
            panic!("{}", String::from_utf8(output.stderr).expect(""));
          }
          
          let output_string = String::from_utf8(output.stdout).expect("pg_dump output is not valid UTF-8 text");
          match action {
            DumpAction::Send(x) => {x.send(output_string).ok();},
            DumpAction::Compare(x, name) => {
              if let Ok(before) = x.recv() {
                if let Some(e) = check_dump_diff(before, output_string, &name) {
                  panic!("{e}");
                }
              }
            }
          }
        }
      }){
        error_t.send(e).ok();
      }));
    }

    Ok(DiffChecker {snapshot_conn,handles,snapshot_sender:Some(tx),error:error_r})
  }

  fn check_err(&mut self) {
    if let Ok(e) = self.error.try_recv() {
      std::panic::resume_unwind(e);
    }
  }

  pub fn finish(&mut self) {
    self.snapshot_sender.take(); // stop threads from waiting
    for handle in self.handles.drain(..) {
      handle.join().expect("");
    }
    self.check_err();
  }

  fn get_snapshot(&mut self) -> String {
    diesel::select(pg_export_snapshot())
      .get_result::<String>(&mut self.snapshot_conn)
      .expect("pg_export_snapshot failed")
  }

  pub fn get_dump(&mut self) -> Receiver<String> {
    self.check_err();
    let snapshot = self.get_snapshot();
    let (tx, rx) = crossbeam_channel::unbounded(); // ::bounded(1);
    self.snapshot_sender.as_mut().expect("").send((snapshot, DumpAction::Send(tx))).expect("send msg");
    rx
  }

  pub fn check_dump_diff(&mut self, before: Receiver<String>, name: String) {
    self.check_err();
    let snapshot = self.get_snapshot();
    self.snapshot_sender.as_mut().expect("").send((snapshot, DumpAction::Compare(before, name))).expect("compare msg");
  }
}


const PATTERN_LEN: usize = 19;

// TODO add unit test for output
pub fn check_dump_diff(mut before: String, mut after: String, name: &str) -> Option<String> {
  if after == before {
    return None;
  }
  // Ignore timestamp differences by removing timestamps
  for dump in [&mut before, &mut after] {
    for index in 0.. {
      // Check for this pattern: 0000-00-00 00:00:00
      let Some((
        &[a0, a1, a2, a3, b0, a4, a5, b1, a6, a7, b2, a8, a9, b3, a10, a11, b4, a12, a13],
        remaining,
      )) = dump
        .get(index..)
        .and_then(|s| s.as_bytes().split_first_chunk::<PATTERN_LEN>())
      else {
        break;
      };
      if [a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13]
        .into_iter()
        .all(|byte| byte.is_ascii_digit())
        && [b0, b1, b2, b3, b4] == *b"-- ::"
      {
        // Replace the part of the string that has the checked pattern and an optional fractional part
        let len_after = if let Some((b'.', s)) = remaining.split_first() {
          1 + s.iter().position(|c| !c.is_ascii_digit()).unwrap_or(0)
        } else {
          0
        };
        // Length of replacement string is likely to match previous string
        // (usually there's up to 6 digits after the decimal point)
        dump.replace_range(
          index..(index + PATTERN_LEN + len_after),
          "AAAAAAAAAAAAAAAAAAAAAAAAAA",
        );
      }
    }
  }

  let [before_chunks, after_chunks] =
    [&before, &after].map(|dump| chunks(dump).collect::<BTreeSet<_>>());

  // todo dont collect only_in_before?
  let [mut only_in_before, mut only_in_after] = [
    before_chunks.difference(&after_chunks),
    after_chunks.difference(&before_chunks),
  ]
  .map(|chunks| {
    chunks
      .map(|chunk| {
        (
          &**chunk,
          // Used for ignoring formatting differences, especially indentation level, when
          // determining which item in `only_in_before` corresponds to which item in `only_in_after`
          chunk.replace([' ', '\t', '\r', '\n'], ""),
        )
      })
      .collect::<Vec<_>>()
  });
  if only_in_before.is_empty() && only_in_after.is_empty() {
    return None;
  }
  let after_has_more =
  only_in_before.len() < only_in_after.len();
  // outer iterator in the loop below should not be the one with empty strings, otherwise the empty strings
  // would be equally similar to any other chunk
  let (chunks_gt, chunks_lt) = if after_has_more
  {
    only_in_before.resize_with(only_in_after.len(),Default::default);
    (&mut only_in_after, &only_in_before)
  } else {
    only_in_after.resize_with(only_in_before.len(),Default::default);
    (&mut only_in_before, &only_in_after)
  };

  let mut output = format!("These changes need to be applied in {name}:");
  // todo rename variables
  for (before_chunk, before_chunk_filtered) in chunks_lt {
    let default = Default::default();
    //panic!("{:?}",(before_chunk.clone(),chunks_lt.clone()));
    let (most_similar_chunk_index, (most_similar_chunk, most_similar_chunk_filtered)) = chunks_gt
      .iter()
      .enumerate()
      .max_by_key(|(_, (after_chunk, after_chunk_filtered))| {
        diff::chars(after_chunk_filtered, &before_chunk_filtered)
          .into_iter()
          .filter(|i| matches!(i, diff::Result::Both(c, _)
          // `is_lowercase` increases accuracy for some trigger function diffs
          if c.is_lowercase() || c.is_numeric()))
          .count()
      })
      .unwrap_or((0,&default));

    output.push('\n');
    let lines = if !after_has_more{diff::lines(&before_chunk,most_similar_chunk)}else{
    diff::lines(most_similar_chunk, &before_chunk)};
    for line in lines
    {
      match line {
        diff::Result::Left(s) => write!(&mut output, "\n- {s}"),
        diff::Result::Right(s) => write!(&mut output, "\n+ {s}"),
        diff::Result::Both(s, _) => write!(&mut output, "\n  {s}"),
      }
      .expect("failed to build string");
    }
    write!(&mut output, "\n{most_similar_chunk_filtered}").expect("");
    if !chunks_gt.is_empty() {
    chunks_gt.swap_remove(most_similar_chunk_index);}
  }
  // should have all been removed
  assert_eq!(chunks_gt.len(), 0);
  Some(output)
}

// todo inline?
fn chunks<'a>(dump: &'a str) -> impl Iterator<Item = Cow<'a, str>> {
  let mut remaining = dump;
  std::iter::from_fn(move || {
    remaining = remaining.trim_start();
    while let Some(s) = remove_skipped_item_from_beginning(remaining) {
      remaining = s.trim_start();
    }
    // `a` can't be empty because of trim_start
    let (result, after_result) = remaining.split_once("\n\n")?;
    remaining = after_result;
    Some(if result.starts_with("CREATE TABLE ") {
      // Allow column order to change
      let mut lines = result
        .lines()
        .map(|line| line.strip_suffix(',').unwrap_or(line))
        .collect::<Vec<_>>();
      lines.sort_unstable_by_key(|line| -> (u8, &str) {
        let placement = match line.chars().next() {
          Some('C') => 0,
          Some(' ') => 1,
          Some(')') => 2,
          _ => panic!("unrecognized part of `CREATE TABLE` statement: {line}"),
        };
        (placement, line)
      });
      Cow::Owned(lines.join("\n"))
    } else if result.starts_with("CREATE VIEW") || result.starts_with("CREATE OR REPLACE VIEW") {
      // Allow column order to change
      let is_simple_select_statement = result
        .lines()
        .enumerate()
        .all(|(i, mut line)| {
          line = line.trim_start();
          match (i, line.chars().next()) {
            (0, Some('C')) => true, // create
            (1, Some('S')) => true, // select
            (_, Some('F')) if line.ends_with(';') => true, // from
            (_, Some(c)) if c.is_lowercase() => true, // column name
            _ => false
          }
        });
      if is_simple_select_statement {
        let mut lines = result
          .lines()
          .map(|line| line.strip_suffix(',').unwrap_or(line))
          .collect::<Vec<_>>();
        lines.sort_unstable_by_key(|line| -> (u8, &str) {
          let placement = match line.trim_start().chars().next() {
            Some('C') => 0,
            Some('S') => 1,
            Some('F') => 3,
            _ => 2,
          };
          (placement, line)
        });
        Cow::Owned(lines.join("\n"))
      }else{Cow::Borrowed(result)}
    } else {
      Cow::Borrowed(result)
    })
  })
}

fn remove_skipped_item_from_beginning(s: &str) -> Option<&str> {
  // Skip commented line
  if let Some(after) = s.strip_prefix("--") {
    Some(after.split_once('\n').unwrap_or_default().1)
  }
  // Skip view definition that's replaced later (the first definition selects all nulls)
  else if let Some(after) = s.strip_prefix("CREATE VIEW ") {
    let (name, after_name) = after.split_once(' ').unwrap_or_default();
    Some(after_name.split_once("\n\n").unwrap_or_default().1)
      .filter(|after_view| after_view.contains(&format!("\nCREATE OR REPLACE VIEW {name} ")))
  } else {
    None
  }
}
