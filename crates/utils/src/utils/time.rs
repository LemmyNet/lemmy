use chrono::{DateTime, Utc, TimeZone};

pub fn naive_from_unix(time: i64) -> DateTime<Utc> {
  Utc.timestamp_opt(time, 0).single().expect("convert datetime")
}

pub fn convert_datetime(datetime: DateTime<Utc>) -> DateTime<Utc> {
  datetime.into()
}
