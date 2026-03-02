use cfg_if::cfg_if;

fn main() {
  cfg_if! {
    if #[cfg(feature = "full")] {
      println!("{}", config_to_string())
    } else {
    }
  }
}

#[cfg(feature = "full")]
fn config_to_string() -> String {
  use doku::json::{AutoComments, CommentsStyle, Formatting, ObjectsStyle};
  use lemmy_utils::settings::structs::Settings;
  let fmt = Formatting {
    auto_comments: AutoComments::none(),
    comments_style: CommentsStyle {
      separator: "#".to_owned(),
    },
    objects_style: ObjectsStyle {
      surround_keys_with_quotes: false,
      use_comma_as_separator: false,
    },
    ..Default::default()
  };
  doku::to_json_fmt_val(&fmt, &Settings::default())
}

#[cfg(test)]
mod test {
  use crate::config_to_string;

  #[test]
  fn test_config_defaults_updated() -> lemmy_utils::error::LemmyResult<()> {
    let current_config = std::fs::read_to_string("../../config/defaults.hjson")?;
    let mut updated_config = config_to_string();
    updated_config.push('\n');
    if current_config != updated_config {
      let diff = unified_diff::diff(
        current_config.as_bytes(),
        "current",
        updated_config.as_bytes(),
        "expected",
        3,
      );
      panic!("{}", String::from_utf8_lossy(&diff));
    }
    Ok(())
  }
}
