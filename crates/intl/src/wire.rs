//! How a parameter value is written down: the JSON the boundary has always
//! taken, and now the one shape a narrative plan crosses in.
//!
//! Text, a number and a list are themselves; everything the language has that
//! JSON does not is a single `$`-tagged object:
//!
//! ```json
//! { "graha": {"$entity": "graha.SUN"},
//!   "born":  {"$date": {"calendar": "GREGORIAN", "year": 2026, "month": 9, "day": 6}},
//!   "at":    {"$time": {"hour": 6, "minute": 15, "second": 0}},
//!   "when":  {"$datetime": {"date": {…}, "time": {…}}},
//!   "since": {"$ghati": {"ghati": 12, "pala": 30, "vipala": 0}} }
//! ```
//!
//! One shape and one parser: the C boundary's `ts_intl_render` reads its
//! `params_json` through this, so a consumer writing parameters and a
//! composer writing a plan cannot be told apart by the reader.
//!
//! ```
//! use teistro_intl::Value;
//!
//! let written = serde_json::to_string(&Value::entity("graha.SUN"))?;
//! assert_eq!(written, r#"{"$entity":"graha.SUN"}"#);
//! assert_eq!(serde_json::from_str::<Value>(&written)?, Value::entity("graha.SUN"));
//! # Ok::<(), serde_json::Error>(())
//! ```

use core::fmt;

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use teistro_calendar::CalendarDate;
use teistro_core::catalogue::Calendar;

use crate::render::{ClockTime, Value};

/// The tags an object may carry, for the sentence a refusal ends with.
const TAGS: &str = "$entity, $date, $time, $datetime, $ghati";

/// A date as a parameter carries it: the four fields that define it. An era
/// and a resolution are a *reading* of a date and not one, and the boundary
/// has never taken them here.
#[derive(Serialize, Deserialize)]
struct Date {
    calendar: String,
    year: i32,
    month: u8,
    day: u8,
}

impl From<&CalendarDate> for Date {
    fn from(date: &CalendarDate) -> Date {
        Date {
            calendar: String::from(date.calendar.key()),
            year: date.year,
            month: date.month,
            day: date.day,
        }
    }
}

impl Date {
    fn date<E: de::Error>(self) -> Result<CalendarDate, E> {
        let calendar = Calendar::from_key(&self.calendar).ok_or_else(|| {
            de::Error::custom(format!(
                "`calendar` must be a calendar key such as `GREGORIAN`, not `{}`",
                self.calendar
            ))
        })?;
        Ok(CalendarDate::defined(
            calendar, self.year, self.month, self.day,
        ))
    }
}

/// A date and a time together, as `$datetime` carries them.
#[derive(Serialize, Deserialize)]
struct DateTime {
    date: Date,
    time: ClockTime,
}

/// Writes a one-entry object, which is what every tag is.
fn tagged<S: Serializer, T: Serialize>(
    serializer: S,
    tag: &str,
    value: &T,
) -> Result<S::Ok, S::Error> {
    let mut map = serializer.serialize_map(Some(1))?;
    map.serialize_entry(tag, value)?;
    map.end()
}

impl Serialize for Value {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Value::Str(text) => serializer.serialize_str(text),
            Value::Int(whole) => serializer.serialize_i64(*whole),
            Value::Num(number) => serializer.serialize_f64(*number),
            Value::List(items) => items.serialize(serializer),
            Value::Entity(key) => tagged(serializer, "$entity", key),
            Value::Date(date) => tagged(serializer, "$date", &Date::from(date)),
            Value::Time(time) => tagged(serializer, "$time", time),
            Value::DateTime(date, time) => tagged(
                serializer,
                "$datetime",
                &DateTime {
                    date: Date::from(date),
                    time: *time,
                },
            ),
            Value::Ghati(ghati) => tagged(serializer, "$ghati", ghati),
        }
    }
}

/// Reads one value, in the shape this module's documentation gives.
struct Read;

impl<'de> Visitor<'de> for Read {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "text, a number, a list, or an object tagged {TAGS}")
    }

    fn visit_str<E: de::Error>(self, text: &str) -> Result<Value, E> {
        Ok(Value::Str(String::from(text)))
    }

    fn visit_i64<E: de::Error>(self, whole: i64) -> Result<Value, E> {
        Ok(Value::Int(whole))
    }

    fn visit_u64<E: de::Error>(self, whole: u64) -> Result<Value, E> {
        i64::try_from(whole).map_or_else(
            |_| {
                Err(de::Error::custom(
                    "a number outside the range of an integer or a double",
                ))
            },
            |whole| Ok(Value::Int(whole)),
        )
    }

    fn visit_f64<E: de::Error>(self, number: f64) -> Result<Value, E> {
        Ok(Value::Num(number))
    }

    fn visit_bool<E: de::Error>(self, _: bool) -> Result<Value, E> {
        Err(de::Error::custom(
            "booleans and null are not parameter values",
        ))
    }

    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        self.visit_bool(false)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut items: A) -> Result<Value, A::Error> {
        let mut list = Vec::with_capacity(items.size_hint().unwrap_or_default());
        while let Some(item) = items.next_element()? {
            list.push(item);
        }
        Ok(Value::List(list))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut entries: A) -> Result<Value, A::Error> {
        let Some(tag) = entries.next_key::<String>()? else {
            return Err(de::Error::custom(
                "an object must have exactly one `$`-tagged key",
            ));
        };
        let value = match tag.as_str() {
            "$entity" => Value::entity(&entries.next_value::<String>()?),
            "$date" => Value::Date(entries.next_value::<Date>()?.date()?),
            "$time" => Value::Time(entries.next_value()?),
            "$datetime" => {
                let DateTime { date, time } = entries.next_value()?;
                Value::DateTime(date.date()?, time)
            }
            "$ghati" => Value::Ghati(entries.next_value()?),
            other => {
                return Err(de::Error::custom(format!(
                    "`{other}` is not a tag; the tags are {TAGS}"
                )));
            }
        };
        if entries.next_key::<String>()?.is_some() {
            return Err(de::Error::custom(
                "an object must have exactly one `$`-tagged key",
            ));
        }
        Ok(value)
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Value, D::Error> {
        deserializer.deserialize_any(Read)
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        reason = "tests unwrap what they write and fail by panicking"
    )]

    use super::*;
    use crate::render::Ghati;
    use teistro_core::catalogue::Graha;

    fn round(value: &Value) -> Value {
        let written = serde_json::to_string(value).unwrap();
        serde_json::from_str(&written).unwrap_or_else(|err| panic!("{written}: {err}"))
    }

    #[test]
    fn every_kind_of_value_writes_and_reads_back() {
        let date = CalendarDate::defined(Calendar::Gregorian, 2026, 9, 6);
        let time = ClockTime::new(6, 15, 0);
        for value in [
            Value::Str(String::from("text")),
            Value::Int(7),
            Value::Num(1.5),
            Value::List(vec![Value::Int(1), Value::Str(String::from("a"))]),
            Value::catalogued(Graha::Sun),
            Value::Date(date.clone()),
            Value::Time(time),
            Value::DateTime(date, time),
            Value::Ghati(Ghati::new(12, 30, 0)),
        ] {
            assert_eq!(round(&value), value);
        }
    }

    #[test]
    fn the_written_shape_is_the_boundary_s_own() {
        let written = serde_json::to_string(&Value::List(vec![
            Value::Int(1),
            Value::Str(String::from("a")),
            Value::catalogued(Graha::Sun),
        ]))
        .unwrap();
        assert_eq!(written, r#"[1,"a",{"$entity":"graha.SUN"}]"#);
        let date = serde_json::to_string(&Value::Date(CalendarDate::defined(
            Calendar::BikramSambat,
            2083,
            5,
            21,
        )))
        .unwrap();
        assert_eq!(
            date,
            r#"{"$date":{"calendar":"BIKRAM_SAMBAT","year":2083,"month":5,"day":21}}"#
        );
    }

    #[test]
    fn what_is_not_a_value_is_refused_by_name() {
        for (json, what) in [
            (r#"{"$nope": 1}"#, "`$nope` is not a tag"),
            (r#"{"$entity": "graha.SUN", "$time": 1}"#, "exactly one"),
            ("true", "booleans and null"),
            ("null", "booleans and null"),
            (
                r#"{"$date": {"calendar": "NOPE", "year": 1, "month": 1, "day": 1}}"#,
                "must be a calendar key",
            ),
        ] {
            let refusal = serde_json::from_str::<Value>(json).unwrap_err().to_string();
            assert!(refusal.contains(what), "`{json}` said `{refusal}`");
        }
    }
}
