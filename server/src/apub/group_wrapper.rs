use crate::to_datetime_utc;
use activitypub::actor::Group;
use chrono::{DateTime, NaiveDateTime};
use failure::Error;
use serde_json::Value;

pub trait GroupHelper {
  // TODO: id really needs to be a url
  fn set_id(group: &mut Group, id: i32);
  fn get_id(group: &Group) -> Result<i32, Error>;

  fn set_title(group: &mut Group, title: &str);
  fn get_title(group: &Group) -> Result<String, Error>;

  fn set_description(group: &mut Group, description: &Option<String>);
  fn get_description(group: &Group) -> Result<Option<String>, Error>;

  // TODO: also needs to be changed to url
  fn set_creator_id(group: &mut Group, creator_id: i32);
  fn get_creator_id(group: &Group) -> Result<i32, Error>;

  fn set_published(group: &mut Group, published: NaiveDateTime);
  fn get_published(group: &Group) -> Result<NaiveDateTime, Error>;

  fn set_updated(group: &mut Group, updated: Option<NaiveDateTime>);
  fn get_updated(group: &Group) -> Result<Option<NaiveDateTime>, Error>;
}

// TODO: something is crashing and not reporting the error
impl GroupHelper for Group {
  fn set_id(group: &mut Group, id: i32) {
    group.object_props.id = Some(Value::String(id.to_string()));
  }
  fn get_id(group: &Group) -> Result<i32, Error> {
    Ok(get_string_value(group.clone().object_props.id).parse::<i32>()?)
  }

  fn set_title(group: &mut Group, title: &str) {
    group.object_props.name = Some(Value::String(title.to_string()));
  }
  fn get_title(group: &Group) -> Result<String, Error> {
    Ok(get_string_value(group.to_owned().object_props.name))
  }

  fn set_description(group: &mut Group, description: &Option<String>) {
    group.object_props.summary = description.as_ref().map(|d| Value::String(d.to_string()));
  }
  fn get_description(group: &Group) -> Result<Option<String>, Error> {
    Ok(get_string_value_opt(group.to_owned().object_props.summary))
  }

  fn set_creator_id(group: &mut Group, creator_id: i32) {
    group.object_props.attributed_to = Some(Value::String(creator_id.to_string()));
  }
  fn get_creator_id(group: &Group) -> Result<i32, Error> {
    Ok(get_string_value(group.clone().object_props.attributed_to).parse::<i32>()?)
  }

  fn set_published(group: &mut Group, published: NaiveDateTime) {
    group.object_props.published = Some(Value::String(to_datetime_utc(published).to_string()))
  }
  fn get_published(group: &Group) -> Result<NaiveDateTime, Error> {
    let str = get_string_value(group.to_owned().object_props.published);
    // TODO: no idea which date format
    let date = DateTime::parse_from_rfc2822(&str)?;
    Ok(date.naive_local())
  }

  fn set_updated(group: &mut Group, updated: Option<NaiveDateTime>) {
    group.object_props.updated = updated.map(|u| Value::String(u.to_string()));
  }
  fn get_updated(group: &Group) -> Result<Option<NaiveDateTime>, Error> {
    let str = get_string_value_opt(group.to_owned().object_props.updated);
    match str {
      Some(s) => Ok(Some(DateTime::parse_from_rfc2822(&s)?.naive_local())),
      None => Ok(None),
    }
  }
}

fn get_string_value_opt(value: Option<Value>) -> Option<String> {
  value
    .as_ref()
    .map(Value::as_str)
    .flatten()
    .map(str::to_string)
}

fn get_string_value(value: Option<Value>) -> String {
  get_string_value_opt(value).unwrap()
}
