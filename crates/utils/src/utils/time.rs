use chrono::{DateTime, FixedOffset, NaiveDateTime};

pub fn naive_from_unix(time: i64) -> NaiveDateTime {
  NaiveDateTime::from_timestamp_opt(time, 0).expect("convert datetime")
}

pub fn convert_datetime(datetime: NaiveDateTime) -> DateTime<FixedOffset> {
  DateTime::<FixedOffset>::from_utc(
    datetime,
    FixedOffset::east_opt(0).expect("create fixed offset"),
  )
}
