use cfg_if::cfg_if;

fn main() {
  cfg_if! {
    if #[cfg(feature = "full")] {
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
      println!("{}", doku::to_json_fmt_val(&fmt, &Settings::default()));
    } else {

    }
  }
}
