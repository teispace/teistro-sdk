//! The canonical form written straight from a value's `Serialize`, with no
//! `serde_json::Value` between.
//!
//! The form is the one [`super::canonical_json`] has always written: each
//! object's members in code-point order of their keys, a duplicated key
//! keeping its last value, numbers in the grammar of [`super::decimal`], a
//! float JSON cannot hold written `null`, and strings with the escapes
//! JSON requires. It was written from a `Value` tree, and the tree was the
//! cost: a full chart's document built 34 MB of it and spent 36 of the
//! chart's 46 milliseconds there. Here an object's members are written
//! straight into the text as they arrive, and an object whose keys did
//! not arrive in order is put in order when it closes, through one
//! scratch buffer every object shares. So what is held at once is the
//! text and a copy of its largest unordered object, and an object already
//! in order costs no copy at all.
//!
//! Each choice below is the one `serde_json::to_value` makes, so the bytes
//! are the bytes the tree gave; the tests hold the two writers to each
//! other.

use core::fmt::{self, Write as _};

use serde::ser::{self, Impossible, Serialize};

use super::{Digits, push_decimal, push_string};

/// A value written in the canonical form.
///
/// # Errors
///
/// What `serde_json::to_value` refuses: a map key that is not a string
/// or a number, a float key that is not finite, and an integer wider than
/// 64 bits.
pub(super) fn write<T: Serialize + ?Sized>(
    value: &T,
    digits: Digits,
) -> Result<String, Unwritable> {
    let mut out = String::new();
    let mut work = Workspace::default();
    value.serialize(Sink {
        out: &mut out,
        work: &mut work,
        digits,
    })?;
    Ok(out)
}

/// What serde reported it could not write.
#[derive(Debug)]
pub(super) struct Unwritable(String);

impl fmt::Display for Unwritable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Unwritable {}

impl ser::Error for Unwritable {
    fn custom<T: fmt::Display>(message: T) -> Unwritable {
        Unwritable(message.to_string())
    }
}

fn key_must_be_a_string() -> Unwritable {
    Unwritable(String::from("key must be a string"))
}

/// What the writer reuses from object to object.
#[derive(Default)]
struct Workspace {
    /// Members lists no open object is using, for the next one to take.
    pool: Vec<Members>,
    /// An unordered object's text while it is written back in order.
    /// One serves every object: a nested object closes before the one
    /// holding it.
    scratch: String,
}

/// An object's members as they arrive: a map's keys, unescaped, in
/// `keys`, and where each member's `"key":value` lies in the text.
#[derive(Default)]
struct Members {
    keys: String,
    entries: Vec<Member>,
}

struct Member {
    key: Key,
    from: usize,
    to: usize,
}

/// A member's key as the order reads it.
fn key<'k>(keys: &'k str, member: &Member) -> &'k str {
    match member.key {
        Key::Field(name) => name,
        Key::Written(from, to) => keys.get(from..to).unwrap_or_default(),
    }
}

/// A struct's field names are static; a map's keys are written.
#[derive(Clone, Copy)]
enum Key {
    Field(&'static str),
    Written(usize, usize),
}

/// Writes one value where it is pointed.
struct Sink<'a> {
    out: &'a mut String,
    work: &'a mut Workspace,
    digits: Digits,
}

impl<'a> Sink<'a> {
    fn integer(self, value: impl fmt::Display) {
        let _ = write!(self.out, "{value}");
    }

    fn float(self, value: f64) {
        if value.is_finite() {
            push_decimal(value, self.digits, self.out);
        } else {
            self.out.push_str("null");
        }
    }

    fn object(self, variant: Option<&'static str>) -> Object<'a> {
        if let Some(variant) = variant {
            self.out.push('{');
            push_string(variant, self.out);
            self.out.push(':');
        }
        self.out.push('{');
        let mut members = self.work.pool.pop().unwrap_or_default();
        members.keys.clear();
        members.entries.clear();
        Object {
            body: self.out.len(),
            out: self.out,
            work: self.work,
            digits: self.digits,
            members,
            variant: variant.is_some(),
            pending: None,
        }
    }

    fn array(self, variant: Option<&'static str>) -> Array<'a> {
        if let Some(variant) = variant {
            self.out.push('{');
            push_string(variant, self.out);
            self.out.push(':');
        }
        self.out.push('[');
        Array {
            out: self.out,
            work: self.work,
            digits: self.digits,
            first: true,
            variant: variant.is_some(),
        }
    }
}

impl<'a> ser::Serializer for Sink<'a> {
    type Ok = ();
    type Error = Unwritable;
    type SerializeSeq = Array<'a>;
    type SerializeTuple = Array<'a>;
    type SerializeTupleStruct = Array<'a>;
    type SerializeTupleVariant = Array<'a>;
    type SerializeMap = Object<'a>;
    type SerializeStruct = Object<'a>;
    type SerializeStructVariant = Object<'a>;

    fn serialize_bool(self, value: bool) -> Result<(), Unwritable> {
        self.out.push_str(if value { "true" } else { "false" });
        Ok(())
    }

    fn serialize_i8(self, value: i8) -> Result<(), Unwritable> {
        self.integer(value);
        Ok(())
    }

    fn serialize_i16(self, value: i16) -> Result<(), Unwritable> {
        self.integer(value);
        Ok(())
    }

    fn serialize_i32(self, value: i32) -> Result<(), Unwritable> {
        self.integer(value);
        Ok(())
    }

    fn serialize_i64(self, value: i64) -> Result<(), Unwritable> {
        self.integer(value);
        Ok(())
    }

    fn serialize_i128(self, value: i128) -> Result<(), Unwritable> {
        if i64::try_from(value).is_ok() || u64::try_from(value).is_ok() {
            self.integer(value);
            Ok(())
        } else {
            Err(Unwritable(String::from("number out of range")))
        }
    }

    fn serialize_u8(self, value: u8) -> Result<(), Unwritable> {
        self.integer(value);
        Ok(())
    }

    fn serialize_u16(self, value: u16) -> Result<(), Unwritable> {
        self.integer(value);
        Ok(())
    }

    fn serialize_u32(self, value: u32) -> Result<(), Unwritable> {
        self.integer(value);
        Ok(())
    }

    fn serialize_u64(self, value: u64) -> Result<(), Unwritable> {
        self.integer(value);
        Ok(())
    }

    fn serialize_u128(self, value: u128) -> Result<(), Unwritable> {
        if u64::try_from(value).is_ok() {
            self.integer(value);
            Ok(())
        } else {
            Err(Unwritable(String::from("number out of range")))
        }
    }

    fn serialize_f32(self, value: f32) -> Result<(), Unwritable> {
        self.float(f64::from(value));
        Ok(())
    }

    fn serialize_f64(self, value: f64) -> Result<(), Unwritable> {
        self.float(value);
        Ok(())
    }

    fn serialize_char(self, value: char) -> Result<(), Unwritable> {
        push_string(value.encode_utf8(&mut [0; 4]), self.out);
        Ok(())
    }

    fn serialize_str(self, value: &str) -> Result<(), Unwritable> {
        push_string(value, self.out);
        Ok(())
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<(), Unwritable> {
        let mut array = self.array(None);
        for byte in value {
            ser::SerializeSeq::serialize_element(&mut array, byte)?;
        }
        ser::SerializeSeq::end(array)
    }

    fn serialize_none(self) -> Result<(), Unwritable> {
        self.serialize_unit()
    }

    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<(), Unwritable> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<(), Unwritable> {
        self.out.push_str("null");
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), Unwritable> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
    ) -> Result<(), Unwritable> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), Unwritable> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<(), Unwritable> {
        self.out.push('{');
        push_string(variant, self.out);
        self.out.push(':');
        value.serialize(Sink {
            out: &mut *self.out,
            work: &mut *self.work,
            digits: self.digits,
        })?;
        self.out.push('}');
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Array<'a>, Unwritable> {
        Ok(self.array(None))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Array<'a>, Unwritable> {
        Ok(self.array(None))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Array<'a>, Unwritable> {
        Ok(self.array(None))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Array<'a>, Unwritable> {
        Ok(self.array(Some(variant)))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Object<'a>, Unwritable> {
        Ok(self.object(None))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Object<'a>, Unwritable> {
        Ok(self.object(None))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Object<'a>, Unwritable> {
        Ok(self.object(Some(variant)))
    }
}

/// An array, written straight through: its order is its own.
struct Array<'a> {
    out: &'a mut String,
    work: &'a mut Workspace,
    digits: Digits,
    first: bool,
    /// Closes a variant's `{"name":` as well.
    variant: bool,
}

impl Array<'_> {
    fn element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Unwritable> {
        value.serialize(self.next())
    }

    /// Everything an element does but write itself, kept out of the
    /// generic `element` so each type serialised costs only its call.
    #[inline(never)]
    fn next(&mut self) -> Sink<'_> {
        if !self.first {
            self.out.push(',');
        }
        self.first = false;
        Sink {
            out: &mut *self.out,
            work: &mut *self.work,
            digits: self.digits,
        }
    }

    fn close(self) {
        self.out.push(']');
        if self.variant {
            self.out.push('}');
        }
    }
}

impl ser::SerializeSeq for Array<'_> {
    type Ok = ();
    type Error = Unwritable;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Unwritable> {
        self.element(value)
    }

    fn end(self) -> Result<(), Unwritable> {
        self.close();
        Ok(())
    }
}

impl ser::SerializeTuple for Array<'_> {
    type Ok = ();
    type Error = Unwritable;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Unwritable> {
        self.element(value)
    }

    fn end(self) -> Result<(), Unwritable> {
        self.close();
        Ok(())
    }
}

impl ser::SerializeTupleStruct for Array<'_> {
    type Ok = ();
    type Error = Unwritable;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Unwritable> {
        self.element(value)
    }

    fn end(self) -> Result<(), Unwritable> {
        self.close();
        Ok(())
    }
}

impl ser::SerializeTupleVariant for Array<'_> {
    type Ok = ();
    type Error = Unwritable;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Unwritable> {
        self.element(value)
    }

    fn end(self) -> Result<(), Unwritable> {
        self.close();
        Ok(())
    }
}

/// An object: its members written as they come, then put in key order
/// when they did not come in it.
struct Object<'a> {
    out: &'a mut String,
    work: &'a mut Workspace,
    digits: Digits,
    members: Members,
    /// Where its members begin in the text, after its `{`.
    body: usize,
    /// Closes a variant's `{"name":` as well.
    variant: bool,
    /// A map key written and waiting for its value.
    pending: Option<Key>,
}

impl Object<'_> {
    fn member<T: Serialize + ?Sized>(&mut self, key: Key, value: &T) -> Result<(), Unwritable> {
        let from = self.open(key);
        value.serialize(Sink {
            out: &mut *self.out,
            work: &mut *self.work,
            digits: self.digits,
        })?;
        self.written(key, from);
        Ok(())
    }

    /// Writes a member's `"key":` and says where the member begins: the
    /// half of `member` before its value, kept out of the generic `member`
    /// so each type serialised costs only its call.
    #[inline(never)]
    fn open(&mut self, key: Key) -> usize {
        if !self.members.entries.is_empty() {
            self.out.push(',');
        }
        let from = self.out.len();
        push_string(
            self::key(
                &self.members.keys,
                &Member {
                    key,
                    from,
                    to: from,
                },
            ),
            self.out,
        );
        self.out.push(':');
        from
    }

    /// Records a member written from `from` to where the text now ends.
    #[inline(never)]
    fn written(&mut self, key: Key, from: usize) {
        let to = self.out.len();
        self.members.entries.push(Member { key, from, to });
    }

    fn close(mut self) {
        let Members { keys, entries } = &mut self.members;
        let ordered = entries.windows(2).all(|pair| match pair {
            [one, other] => key(keys, one) < key(keys, other),
            _ => true,
        });
        if !ordered {
            // Stable, so a key's occurrences stay in the order they came
            // and the last of each run is the value a map keeps.
            entries.sort_by(|one, other| key(keys, one).cmp(key(keys, other)));
            entries.dedup_by(|later, earlier| {
                let same = key(keys, later) == key(keys, earlier);
                if same {
                    core::mem::swap(later, earlier);
                }
                same
            });
            // The largest member is left where it was written and the
            // rest are written around it: an object holding a large one (a
            // dasha's periods, a document's dashas) is unordered because
            // its small members came first, and a copy out and back of the
            // whole object cost a chart a 3 MB move at every level.
            let largest = entries
                .iter()
                .enumerate()
                .max_by_key(|(_, entry)| entry.to - entry.from)
                .map_or(0, |(index, _)| index);
            let scratch = &mut self.work.scratch;
            scratch.clear();
            let (mut split, mut kept) = (0, (self.body, self.body));
            for (index, entry) in entries.iter().enumerate() {
                if index > 0 {
                    scratch.push(',');
                }
                if index == largest {
                    split = scratch.len();
                    kept = (entry.from, entry.to);
                    continue;
                }
                scratch.push_str(self.out.get(entry.from..entry.to).unwrap_or_default());
            }
            let (before, after) = scratch.split_at(split);
            self.out.replace_range(kept.1.., after);
            self.out.replace_range(self.body..kept.0, before);
        }
        self.out.push('}');
        if self.variant {
            self.out.push('}');
        }
        self.work.pool.push(core::mem::take(&mut self.members));
    }
}

impl ser::SerializeMap for Object<'_> {
    type Ok = ();
    type Error = Unwritable;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Unwritable> {
        let from = self.members.keys.len();
        key.serialize(KeySink {
            out: &mut self.members.keys,
        })?;
        self.pending = Some(Key::Written(from, self.members.keys.len()));
        Ok(())
    }

    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Unwritable> {
        let key = self
            .pending
            .take()
            .ok_or_else(|| Unwritable(String::from("a map value with no key")))?;
        self.member(key, value)
    }

    fn end(self) -> Result<(), Unwritable> {
        self.close();
        Ok(())
    }
}

impl ser::SerializeStruct for Object<'_> {
    type Ok = ();
    type Error = Unwritable;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Unwritable> {
        self.member(Key::Field(key), value)
    }

    fn end(self) -> Result<(), Unwritable> {
        self.close();
        Ok(())
    }
}

impl ser::SerializeStructVariant for Object<'_> {
    type Ok = ();
    type Error = Unwritable;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Unwritable> {
        self.member(Key::Field(key), value)
    }

    fn end(self) -> Result<(), Unwritable> {
        self.close();
        Ok(())
    }
}

/// A map key's text, unescaped, as `serde_json` spells it: a string as
/// itself, a number or a bool as its digits or word, a unit variant as its
/// name.
struct KeySink<'a> {
    out: &'a mut String,
}

impl KeySink<'_> {
    fn integer(self, value: impl fmt::Display) {
        let _ = write!(self.out, "{value}");
    }

    /// A float key, spelt by `serde_json` itself: its formatter is not
    /// the one numbers are written with, and keys like this are rare.
    fn float<F: Serialize>(self, value: F) -> Result<(), Unwritable> {
        struct OneKey<F>(F);
        // lint: serialised-type-describes-itself (a key's spelling, no document)
        impl<F: Serialize> Serialize for OneKey<F> {
            fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut map = serializer.serialize_map(Some(1))?;
                ser::SerializeMap::serialize_entry(&mut map, &self.0, &())?;
                ser::SerializeMap::end(map)
            }
        }
        match serde_json::to_value(OneKey(value)) {
            Ok(serde_json::Value::Object(map)) => {
                let key = map.keys().next().ok_or_else(key_must_be_a_string)?;
                self.out.push_str(key);
                Ok(())
            }
            Ok(_) => Err(key_must_be_a_string()),
            Err(error) => Err(Unwritable(error.to_string())),
        }
    }
}

impl ser::Serializer for KeySink<'_> {
    type Ok = ();
    type Error = Unwritable;
    type SerializeSeq = Impossible<(), Unwritable>;
    type SerializeTuple = Impossible<(), Unwritable>;
    type SerializeTupleStruct = Impossible<(), Unwritable>;
    type SerializeTupleVariant = Impossible<(), Unwritable>;
    type SerializeMap = Impossible<(), Unwritable>;
    type SerializeStruct = Impossible<(), Unwritable>;
    type SerializeStructVariant = Impossible<(), Unwritable>;

    fn serialize_bool(self, value: bool) -> Result<(), Unwritable> {
        self.out.push_str(if value { "true" } else { "false" });
        Ok(())
    }

    fn serialize_i8(self, value: i8) -> Result<(), Unwritable> {
        self.integer(value);
        Ok(())
    }

    fn serialize_i16(self, value: i16) -> Result<(), Unwritable> {
        self.integer(value);
        Ok(())
    }

    fn serialize_i32(self, value: i32) -> Result<(), Unwritable> {
        self.integer(value);
        Ok(())
    }

    fn serialize_i64(self, value: i64) -> Result<(), Unwritable> {
        self.integer(value);
        Ok(())
    }

    fn serialize_i128(self, value: i128) -> Result<(), Unwritable> {
        self.integer(value);
        Ok(())
    }

    fn serialize_u8(self, value: u8) -> Result<(), Unwritable> {
        self.integer(value);
        Ok(())
    }

    fn serialize_u16(self, value: u16) -> Result<(), Unwritable> {
        self.integer(value);
        Ok(())
    }

    fn serialize_u32(self, value: u32) -> Result<(), Unwritable> {
        self.integer(value);
        Ok(())
    }

    fn serialize_u64(self, value: u64) -> Result<(), Unwritable> {
        self.integer(value);
        Ok(())
    }

    fn serialize_u128(self, value: u128) -> Result<(), Unwritable> {
        self.integer(value);
        Ok(())
    }

    fn serialize_f32(self, value: f32) -> Result<(), Unwritable> {
        self.float(value)
    }

    fn serialize_f64(self, value: f64) -> Result<(), Unwritable> {
        self.float(value)
    }

    fn serialize_char(self, value: char) -> Result<(), Unwritable> {
        self.out.push(value);
        Ok(())
    }

    fn serialize_str(self, value: &str) -> Result<(), Unwritable> {
        self.out.push_str(value);
        Ok(())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<(), Unwritable> {
        Err(key_must_be_a_string())
    }

    fn serialize_none(self) -> Result<(), Unwritable> {
        Err(key_must_be_a_string())
    }

    fn serialize_some<T: Serialize + ?Sized>(self, _value: &T) -> Result<(), Unwritable> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit(self) -> Result<(), Unwritable> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), Unwritable> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
    ) -> Result<(), Unwritable> {
        self.out.push_str(variant);
        Ok(())
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), Unwritable> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<(), Unwritable> {
        Err(key_must_be_a_string())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Unwritable> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Unwritable> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Unwritable> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Unwritable> {
        Err(key_must_be_a_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Unwritable> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Unwritable> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Unwritable> {
        Err(key_must_be_a_string())
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::panic, reason = "tests fail by panicking")]

    use std::collections::{BTreeMap, HashMap};

    use proptest::prelude::*;
    use serde::Serialize;
    use serde::ser::{SerializeMap, Serializer};
    use serde_json::{Map, Number, Value};

    use super::*;
    use crate::envelope::write_canonical;

    /// The two writers agree, to the byte or in refusing.
    fn agree<T: Serialize + ?Sized>(value: &T) {
        for digits in [Digits::Shortest, Digits::Rounded(3)] {
            let streamed = write(value, digits).map_err(|error| error.to_string());
            let tree = serde_json::to_value(value)
                .map(|tree| write_canonical(&tree, digits))
                .map_err(|error| error.to_string());
            match (&streamed, &tree) {
                (Ok(one), Ok(other)) => assert_eq!(one, other),
                (Err(_), Err(_)) => {}
                _ => panic!("one writer refused: streamed {streamed:?}, tree {tree:?}"),
            }
        }
    }

    #[derive(Serialize)]
    struct Unordered {
        zeta: u8,
        alpha: Option<f64>,
        #[serde(rename = "Mixed")]
        mixed: Vec<Shape>,
        nested: Inner,
        #[serde(flatten)]
        flat: BTreeMap<String, i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        skipped: Option<u8>,
    }

    #[derive(Serialize)]
    struct Inner {
        b: char,
        a: (i8, f32, ()),
        unit: Unit,
        wrapped: Wrapped,
    }

    #[derive(Serialize)]
    struct Unit;

    #[derive(Serialize)]
    struct Wrapped(u64);

    #[derive(Serialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    enum Shape {
        Bare,
        Newtype(f64),
        Tuple(i64, String),
        Struct { y: u16, x: bool },
    }

    #[derive(Serialize)]
    #[serde(tag = "kind", rename_all = "lowercase")]
    enum Tagged {
        One { value: f64 },
        Two { list: Vec<u8> },
    }

    /// A map that names one key twice: the last value is the one kept.
    struct Twice;

    impl Serialize for Twice {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut map = serializer.serialize_map(None)?;
            map.serialize_entry("b", &1)?;
            map.serialize_entry("a", &2)?;
            map.serialize_entry("b", &3)?;
            map.end()
        }
    }

    /// Keys of every kind a JSON map accepts, and some it does not.
    struct Keyed<K>(Vec<(K, u8)>);

    impl<K: Serialize> Serialize for Keyed<K> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut map = serializer.serialize_map(Some(self.0.len()))?;
            for (key, value) in &self.0 {
                map.serialize_entry(key, value)?;
            }
            map.end()
        }
    }

    struct Bytes(&'static [u8]);

    impl Serialize for Bytes {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_bytes(self.0)
        }
    }

    #[test]
    fn every_shape_serde_writes_reads_as_the_tree_wrote_it() {
        agree(&Unordered {
            zeta: 7,
            alpha: Some(-0.0),
            mixed: vec![
                Shape::Bare,
                Shape::Newtype(1.0e-7),
                Shape::Tuple(-3, String::from("q\"uote\\ \n\t\u{1}नेपाल")),
                Shape::Struct { y: 9, x: true },
            ],
            nested: Inner {
                b: '"',
                a: (-128, 0.1, ()),
                unit: Unit,
                wrapped: Wrapped(u64::MAX),
            },
            flat: BTreeMap::from([(String::from("aardvark"), 1), (String::from("zeta"), 2)]),
            skipped: None,
        });
        agree(&[Tagged::One { value: 2.5 }, Tagged::Two { list: vec![1, 2] }]);
        agree(&Twice);
        agree(&Bytes(b"\x00\xff"));
        agree(&[
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            -0.0,
            2_460_483.123_456_789,
        ]);
        agree(&[f32::NAN, 1.1, f32::MAX]);
        agree(&(i128::from(i64::MIN), u128::from(u64::MAX)));
        agree(&i128::MIN);
        agree(&u128::MAX);
        agree(&HashMap::from([(3_u8, "c"), (1, "a"), (20, "t")]));
        agree(&Keyed(vec![(true, 1), (false, 2)]));
        agree(&Keyed(vec![(-1_i128, 1), (i128::MAX, 2)]));
        agree(&Keyed(vec![(1.5_f64, 1), (-0.0, 2), (1e300, 3)]));
        agree(&Keyed(vec![(f64::NAN, 1)]));
        agree(&Keyed(vec![('x', 1), ('\u{0}', 2)]));
        agree(&Keyed(vec![(Shape::Bare, 1)]));
        agree(&Keyed(vec![(Some(1), 1)]));
        agree(&Keyed(vec![(vec![1], 1)]));
        agree(&Keyed(vec![((), 1)]));
        agree(&Keyed(vec![(Wrapped(4), 1)]));
        agree(&Value::Null);
        agree(&Vec::<u8>::new());
        agree(&BTreeMap::<String, u8>::new());
    }

    #[derive(Serialize)]
    struct Pair(i16, Option<u8>);

    #[derive(Serialize, PartialEq, Eq, PartialOrd, Ord)]
    enum Variant {
        Unit,
        Newtype(u8),
        Tuple(u8, u8),
        Struct { a: u8 },
    }

    #[derive(Serialize)]
    struct Holds {
        inner: Failing,
    }

    /// A value whose `Serialize` fails on its own account.
    struct Failing;

    impl Serialize for Failing {
        fn serialize<S: Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("this value cannot be written"))
        }
    }

    #[test]
    fn every_key_shape_reads_as_the_tree_wrote_it_or_both_refuse() {
        agree(&Keyed(vec![(-1_i8, 1), (i8::MAX, 2)]));
        agree(&Keyed(vec![(-1_i16, 1), (i16::MIN, 2)]));
        agree(&Keyed(vec![(-1_i32, 1), (i32::MAX, 2)]));
        agree(&Keyed(vec![(-1_i64, 1), (i64::MIN, 2)]));
        agree(&Keyed(vec![(0_u8, 1), (u8::MAX, 2)]));
        agree(&Keyed(vec![(0_u16, 1), (u16::MAX, 2)]));
        agree(&Keyed(vec![(0_u32, 1), (u32::MAX, 2)]));
        agree(&Keyed(vec![(0_u64, 1), (u64::MAX, 2)]));
        agree(&Keyed(vec![(0_u128, 1), (u128::MAX, 2)]));
        agree(&Keyed(vec![(1.5_f32, 1), (-0.0, 2)]));
        agree(&Keyed(vec![("b", 1), ("a", 2)]));
        agree(&Keyed(vec![(Variant::Unit, 1)]));
        agree(&Keyed(vec![(Variant::Newtype(1), 1)]));
        agree(&Keyed(vec![(Variant::Tuple(1, 2), 1)]));
        agree(&Keyed(vec![(Variant::Struct { a: 1 }, 1)]));
        agree(&Keyed(vec![(None::<u8>, 1)]));
        agree(&Keyed(vec![(Unit, 1)]));
        agree(&Keyed(vec![(Pair(1, None), 1)]));
        agree(&Keyed(vec![((1, 2), 1)]));
        agree(&Keyed(vec![(BTreeMap::from([("a", 1)]), 1)]));
        agree(&Keyed(vec![(
            Inner {
                b: 'b',
                a: (0, 0.0, ()),
                unit: Unit,
                wrapped: Wrapped(0),
            },
            1,
        )]));
        agree(&Keyed(vec![(Bytes(b"k"), 1)]));
        agree(&Keyed(vec![(Failing, 1)]));
    }

    /// A key a map writes twice keeps its last value, wherever the
    /// object's largest member lies: before the repeat, the repeat
    /// itself, or a value the repeat replaces.
    #[test]
    fn a_repeated_key_keeps_its_last_value() {
        let written = |keyed: &Keyed<&str>| {
            write(keyed, Digits::Shortest).unwrap_or_else(|error| panic!("{error}"))
        };
        let cases = [
            (vec![("b", 1), ("a", 2), ("b", 3)], r#"{"a":2,"b":3}"#),
            (
                vec![("long", 200), ("a", 1), ("long", 2)],
                r#"{"a":1,"long":2}"#,
            ),
            (
                vec![("c", 1), ("longest", 100), ("a", 2), ("c", 3)],
                r#"{"a":2,"c":3,"longest":100}"#,
            ),
            (vec![("z", 1), ("z", 2), ("z", 3)], r#"{"z":3}"#),
        ];
        for (pairs, expected) in cases {
            let keyed = Keyed(pairs);
            assert_eq!(written(&keyed), expected);
            agree(&keyed);
        }
    }

    #[test]
    fn every_value_shape_reads_as_the_tree_wrote_it_or_both_refuse() {
        agree(&Pair(-7, None));
        agree(&[Pair(1, Some(2)), Pair(i16::MIN, None)]);
        agree(&[
            Variant::Unit,
            Variant::Newtype(3),
            Variant::Tuple(4, 5),
            Variant::Struct { a: 6 },
        ]);
        agree(&(1_i8, 2_i16, 3_i32, 4_u8, 5_u16, 6_u32, None::<u8>, Unit));
        agree(&Failing);
        agree(&[Failing]);
        agree(&Holds { inner: Failing });
    }

    #[test]
    fn a_refusal_says_why() {
        let refused = write(&Failing, Digits::Shortest)
            .map_or_else(|error| error.to_string(), |_| String::new());
        assert_eq!(refused, "this value cannot be written");
        let keyed = write(&Keyed(vec![(vec![1_u8], 1)]), Digits::Shortest)
            .map_or_else(|error| error.to_string(), |_| String::new());
        assert_ne!(keyed, "");
    }

    fn values() -> impl Strategy<Value = Value> {
        let leaf = prop_oneof![
            Just(Value::Null),
            any::<bool>().prop_map(Value::Bool),
            any::<i64>().prop_map(|n| Value::Number(n.into())),
            any::<u64>().prop_map(|n| Value::Number(n.into())),
            any::<f64>().prop_map(|x| Number::from_f64(x).map_or(Value::Null, Value::Number)),
            "\\PC{0,8}".prop_map(Value::String),
            "[\\x00-\\x1f\"\\\\a]{0,4}".prop_map(Value::String),
        ];
        leaf.prop_recursive(4, 64, 8, |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..8).prop_map(Value::Array),
                prop::collection::vec(("\\PC{0,4}", inner), 0..8)
                    .prop_map(|pairs| Value::Object(pairs.into_iter().collect::<Map<_, _>>())),
            ]
        })
    }

    proptest! {
        #[test]
        fn any_tree_reads_as_the_tree_wrote_it(value in values()) {
            agree(&value);
        }
    }
}
