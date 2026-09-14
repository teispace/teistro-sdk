//! What the generated dispatch is built from: reading an argument,
//! checking a status, and nothing else.
//!
//! Hand-written on purpose, and small on purpose. `src/dispatch.rs` is
//! one arm per engine function and is regenerated whenever the engine
//! changes; anything it needs that is the *same* for every arm belongs
//! here, where it is written once and read by a person.

use core::cell::RefCell;
use core::ffi::{CStr, c_char};
use std::ffi::CString;

use serde_json::{Map, Value};
use teistro_port_ephemeris::ProviderError;

/// An argument that must be a number, as a `f64`.
///
/// # Errors
///
/// `Invalid` naming the argument when it is missing or is not a number.
/// Naming it matters: a call through this route is written by hand
/// against a manifest, so the mistake is nearly always a name.
pub(crate) fn number(args: &Map<String, Value>, name: &str) -> Result<f64, ProviderError> {
    number_of(args.get(name), name)
}

/// An argument that must be a whole number, as an `i64`.
///
/// A JSON number with a fractional part is **refused** rather than
/// truncated. The engine's integer parameters are counts, ids and enum
/// members, and silently turning 2.7 into 2 would answer a question the
/// caller did not ask.
///
/// `true` and `false` are accepted for an integer, because C has no
/// separate boolean at this boundary and a caller writing JSON will reach
/// for one where the engine declares a flag.
///
/// # Errors
///
/// `Invalid` naming the argument when it is missing, not a number, or not
/// whole.
#[cfg(test)]
pub(crate) fn integer(args: &Map<String, Value>, name: &str) -> Result<i64, ProviderError> {
    integer_of(args.get(name), name)
}

/// A whole-number argument narrowed to the width the engine declares.
///
/// **Refused rather than truncated.** The engine's integer parameters are
/// counts, ids and enum members, and a caller who passes a number too
/// large for one has made a mistake that silently keeping the low bits
/// would turn into a wrong answer instead of a message.
///
/// # Errors
///
/// `Invalid` naming the argument and the width it did not fit.
pub(crate) fn narrow<T>(args: &Map<String, Value>, name: &str) -> Result<T, ProviderError>
where
    T: TryFrom<i64>,
{
    narrow_of(args.get(name), name)
}

/// An argument that must be an object: a struct, as the engine's own
/// field names key it.
///
/// # Errors
///
/// `Invalid` naming the argument when it is missing or not an object.
#[allow(
    dead_code,
    reason = "the generator emits it for a struct the engine does not mark optional, and this \
              engine marks every struct input optional; kept so the next one compiles"
)]
pub(crate) fn object<'a>(
    args: &'a Map<String, Value>,
    name: &str,
) -> Result<Within<'a>, ProviderError> {
    object_of(args.get(name), name.to_owned())
}

/// An argument that may be an object, or left out, or null — the three
/// ways a caller says "no struct here", which crosses as a null pointer.
///
/// # Errors
///
/// `Invalid` naming the argument when it is present and neither null nor
/// an object: a number where an observer belongs is a mistake, not an
/// absence.
pub(crate) fn optional_object<'a>(
    args: &'a Map<String, Value>,
    name: &str,
) -> Result<Option<Within<'a>>, ProviderError> {
    match args.get(name) {
        None | Some(Value::Null) => Ok(None),
        found => object_of(found, name.to_owned()).map(Some),
    }
}

/// An argument that must be an array of numbers.
///
/// # Errors
///
/// `Invalid` naming the argument when it is missing or not an array, and
/// naming the element — `jds[3]` — when one is not a number.
pub(crate) fn numbers(args: &Map<String, Value>, name: &str) -> Result<Vec<f64>, ProviderError> {
    each(args, name, number_of)
}

/// An argument that must be an array of whole numbers, each narrowed to
/// the width the engine declares and refused rather than truncated.
///
/// # Errors
///
/// As [`numbers`], and naming the element that does not fit.
pub(crate) fn wholes<T>(args: &Map<String, Value>, name: &str) -> Result<Vec<T>, ProviderError>
where
    T: TryFrom<i64>,
{
    each(args, name, |found, path| narrow_of(found, path))
}

/// An argument that must be an array of objects, each read as a struct
/// by `read`.
///
/// # Errors
///
/// As [`numbers`], and naming the element — `dts[2].month` — whose field
/// is missing or wrong.
pub(crate) fn objects<T>(
    args: &Map<String, Value>,
    name: &str,
    read: impl Fn(&Within<'_>) -> Result<T, ProviderError>,
) -> Result<Vec<T>, ProviderError> {
    each(args, name, |found, path| {
        read(&object_of(found, path.to_owned())?)
    })
}

/// Every element of an array argument, read by one rule and named by
/// its index when the rule refuses it.
fn each<T>(
    args: &Map<String, Value>,
    name: &str,
    read: impl Fn(Option<&Value>, &str) -> Result<T, ProviderError>,
) -> Result<Vec<T>, ProviderError> {
    let items = match args.get(name) {
        Some(Value::Array(items)) => items,
        Some(other) => {
            return Err(ProviderError::invalid(format!(
                "`{name}` must be an array; it is {}",
                kind(other)
            )));
        }
        None => return Err(ProviderError::invalid(format!("`{name}` is required"))),
    };
    items
        .iter()
        .enumerate()
        .map(|(at, item)| read(Some(item), &format!("{name}[{at}]")))
        .collect()
}

/// What a struct argument points at, kept alive for the call.
///
/// A struct the engine reads may hold a string or a pointer to another
/// struct, and what it points at has to outlive the call and must not
/// move. `Keep` owns each of them from the moment it is read until the arm
/// that bound it returns.
///
/// **Raw ownership, not a `Vec` of boxes.** A pointer taken from a `Box`
/// or a `CString` is invalidated when the owner is moved — a push that
/// grows the `Vec` moves every element — so each allocation is released
/// into a raw pointer at once (`Box::into_raw`, `CString::into_raw`), and
/// only the raw pointers are stored. They are reclaimed exactly once, in
/// `Drop`.
#[derive(Default)]
pub(crate) struct Keep {
    kept: RefCell<Vec<Kept>>,
}

/// One allocation `Keep` owns.
enum Kept {
    Text(*mut c_char),
    Value {
        at: *mut (),
        free: unsafe fn(*mut ()),
    },
}

impl Keep {
    /// A string, as the pointer the engine reads, valid until `self` drops.
    pub(crate) fn text(&self, text: CString) -> *const c_char {
        let at = text.into_raw();
        self.kept.borrow_mut().push(Kept::Text(at));
        at.cast_const()
    }

    /// A value, as the pointer the engine reads, valid until `self` drops.
    pub(crate) fn value<T: 'static>(&self, value: T) -> *const T {
        let at = Box::into_raw(Box::new(value));
        self.kept.borrow_mut().push(Kept::Value {
            at: at.cast(),
            free: free::<T>,
        });
        at.cast_const()
    }
}

/// Reclaims one value `Keep::value` released.
///
/// # Safety
///
/// `at` must be a pointer `Box::into_raw` returned for a `Box<T>`, not yet
/// reclaimed.
#[allow(unsafe_code, reason = "reclaiming a box released by `Keep::value`")]
unsafe fn free<T>(at: *mut ()) {
    // SAFETY: the caller's contract, which `Keep::drop` keeps by storing
    // the pointer beside the function monomorphised for its own type.
    drop(unsafe { Box::from_raw(at.cast::<T>()) });
}

impl Drop for Keep {
    #[allow(unsafe_code, reason = "reclaiming what `Keep` released, once")]
    fn drop(&mut self) {
        for kept in self.kept.get_mut().drain(..) {
            match kept {
                // SAFETY: released by `CString::into_raw` in `Keep::text`
                // and reclaimed only here.
                Kept::Text(at) => drop(unsafe { CString::from_raw(at) }),
                // SAFETY: released by `Box::into_raw` in `Keep::value` with
                // `free` monomorphised for its type; reclaimed only here.
                Kept::Value { at, free } => unsafe { free(at) },
            }
        }
    }
}

/// A struct the engine points at from inside another, as the object a
/// caller gets back: `null` for a null pointer.
#[allow(
    unsafe_code,
    reason = "reading a struct through a pointer the engine wrote into a struct it filled; the \
              generated dispatch is the only caller"
)]
pub(crate) fn pointed<T>(at: *const T, write: fn(&T) -> Value) -> Value {
    if at.is_null() {
        return Value::Null;
    }
    // SAFETY: a non-null pointer field of a struct the engine filled points
    // at a struct of the declared type, valid for the rest of the call —
    // into the engine's own memory or into what `Keep` holds.
    write(unsafe { &*at })
}

/// The most one output array may take, in bytes.
///
/// A bound and not a tuning knob. The room for an answer is sized by the
/// caller's own inputs, or — for a search — by the number the caller asks
/// for, and a caller who asks for ten billion eclipses must be told so
/// rather than have the process aborted by an allocation that cannot be
/// met. 256 MiB is about three million positions, which is far past
/// anything a JSON answer is a sensible shape for.
const MOST_BYTES: usize = 256 << 20;

/// Room for `length` elements the engine will write, each as `default`
/// makes it.
///
/// `default` and not zeroed, because a struct's default is the binding's
/// own way of declaring its extent, and the engine reads **every
/// element's** `struct_size` to learn the stride it writes at.
///
/// # Errors
///
/// `Invalid` naming the output when the room would pass [`MOST_BYTES`],
/// and `Refused` when the allocation itself fails — both rather than an
/// abort, because the length can come from the caller.
pub(crate) fn room<T>(length: usize, name: &str) -> Result<Vec<T>, ProviderError>
where
    T: Clone + Default,
{
    let bytes = length.saturating_mul(core::mem::size_of::<T>());
    if bytes > MOST_BYTES {
        return Err(ProviderError::invalid(format!(
            "`{name}` would hold {length} elements, {bytes} bytes; this adapter makes at most \
             {MOST_BYTES} bytes of room for one answer"
        )));
    }
    let mut room = Vec::new();
    room.try_reserve_exact(length)
        .map_err(|_| ProviderError::Refused {
            detail: format!("`{name}` could not be given room for {length} elements"),
        })?;
    room.resize(length, T::default());
    Ok(room)
}

/// The length of an output laid out over several inputs — bodies by
/// epochs — as their product.
///
/// # Errors
///
/// `Invalid` naming the output when the product does not fit a `usize`.
pub(crate) fn extent(lengths: &[usize], name: &str) -> Result<usize, ProviderError> {
    lengths
        .iter()
        .try_fold(1_usize, |so_far, length| so_far.checked_mul(*length))
        .ok_or_else(|| {
            ProviderError::invalid(format!(
                "`{name}` would be {} elements long, which no buffer can be",
                lengths
                    .iter()
                    .map(usize::to_string)
                    .collect::<Vec<_>>()
                    .join(" × ")
            ))
        })
}

/// A value an output's length is read from — a field of a request, such
/// as a calendar's `day_count` — as a length.
///
/// # Errors
///
/// `Invalid` naming the value's path when it is negative or too large for
/// a length, rather than casting it into one.
pub(crate) fn length<T>(value: T, path: &str) -> Result<usize, ProviderError>
where
    T: TryInto<usize> + Copy + core::fmt::Display,
{
    value
        .try_into()
        .map_err(|_| ProviderError::invalid(format!("`{path}` is {value}, which is not a length")))
}

/// An output the engine says the length of, by the engine's own
/// protocol: the call writes what fits and reports **how many there
/// are**.
///
/// The same shape as [`fill`] for an array: the first call is made into
/// room for [`ROOMY_ELEMENTS`], which is every answer these functions
/// usually have, and only an answer that did not fit is asked for again
/// into room for exactly what it said. `call` runs at most twice and must
/// be the same call both times.
///
/// # Errors
///
/// Whatever `call` refuses with, and `Refused` when the second call
/// reported more than the first — an answer that changed underneath the
/// protocol, which a caller must be told about rather than handed a
/// truncation.
pub(crate) fn gather<T>(
    function: &str,
    mut call: impl FnMut(&mut [T]) -> Result<usize, ProviderError>,
) -> Result<Vec<T>, ProviderError>
where
    T: Clone + Default,
{
    /// Room for the answers these functions usually give — a chart's
    /// default bodies, the stars sharing a name — in one call.
    const ROOMY_ELEMENTS: usize = 64;

    let first = ROOMY_ELEMENTS.min(MOST_BYTES / core::mem::size_of::<T>().max(1));
    let mut answer = room::<T>(first, function)?;
    let there = call(&mut answer)?;
    if there <= answer.len() {
        answer.truncate(there);
        return Ok(answer);
    }
    let mut answer = room::<T>(there, function)?;
    let again = call(&mut answer)?;
    if again > answer.len() {
        return Err(ProviderError::Refused {
            detail: format!(
                "`{function}` said there were {there} and then {again}; its answer changed \
                 between the two calls"
            ),
        });
    }
    answer.truncate(again);
    Ok(answer)
}

/// The object a struct's fields are read from, and the path a refusal
/// names them by.
///
/// A struct crosses as a nested object, so the key a field is looked up
/// under and the name a message must print are no longer one string: the
/// field is `year` and the caller wrote `local.year`. One type carries
/// both down through the nesting, so every refusal names the whole path.
pub(crate) struct Within<'a> {
    object: &'a Map<String, Value>,
    path: String,
}

impl<'a> Within<'a> {
    /// A field's full path, as the caller wrote it.
    fn at(&self, name: &str) -> String {
        format!("{}.{name}", self.path)
    }

    /// A field that must be a number.
    ///
    /// # Errors
    ///
    /// As [`number`], naming the field's whole path.
    pub(crate) fn number(&self, name: &str) -> Result<f64, ProviderError> {
        number_of(self.object.get(name), &self.at(name))
    }

    /// A whole-number field narrowed to the width the engine declares.
    ///
    /// # Errors
    ///
    /// As [`narrow`], naming the field's whole path.
    pub(crate) fn narrow<T>(&self, name: &str) -> Result<T, ProviderError>
    where
        T: TryFrom<i64>,
    {
        narrow_of(self.object.get(name), &self.at(name))
    }

    /// A field that is itself a struct.
    ///
    /// # Errors
    ///
    /// `Invalid` naming the field's whole path when it is missing or not
    /// an object.
    /// A string field, kept alive in `keep` for the call, or null.
    ///
    /// # Errors
    ///
    /// `Invalid` naming the field's whole path when it is missing, neither
    /// null nor a string, or holds a NUL.
    pub(crate) fn text(&self, name: &str, keep: &Keep) -> Result<*const c_char, ProviderError> {
        let path = self.at(name);
        match nullable_of(self.object.get(name), &path)? {
            None => Ok(core::ptr::null()),
            found => Ok(keep.text(text_of(found, &path)?)),
        }
    }

    /// A field pointing at a struct: the struct, read by `read` and kept
    /// alive in `keep` for the call, or null.
    ///
    /// # Errors
    ///
    /// `Invalid` naming the field's whole path when it is missing, neither
    /// null nor an object, or when `read` refuses one of its fields.
    pub(crate) fn pointer<T: 'static>(
        &self,
        name: &str,
        keep: &Keep,
        read: impl FnOnce(&Within<'_>) -> Result<T, ProviderError>,
    ) -> Result<*const T, ProviderError> {
        let path = self.at(name);
        match nullable_of(self.object.get(name), &path)? {
            None => Ok(core::ptr::null()),
            found => Ok(keep.value(read(&object_of(found, path)?)?)),
        }
    }

    #[allow(
        dead_code,
        reason = "the generator emits it for a struct passed in that holds another by value; no \
                  such function is callable at this engine version"
    )]
    pub(crate) fn nested(&self, name: &str) -> Result<Within<'a>, ProviderError> {
        object_of(self.object.get(name), self.at(name))
    }
}

fn number_of(found: Option<&Value>, path: &str) -> Result<f64, ProviderError> {
    match found {
        Some(Value::Number(found)) => found.as_f64().ok_or_else(|| {
            ProviderError::invalid(format!("`{path}` is a number this engine cannot take"))
        }),
        Some(other) => Err(ProviderError::invalid(format!(
            "`{path}` must be a number; it is {}",
            kind(other)
        ))),
        None => Err(ProviderError::invalid(format!("`{path}` is required"))),
    }
}

fn integer_of(found: Option<&Value>, path: &str) -> Result<i64, ProviderError> {
    match found {
        Some(Value::Bool(flag)) => Ok(i64::from(*flag)),
        Some(Value::Number(found)) => {
            if let Some(whole) = found.as_i64() {
                return Ok(whole);
            }
            if let Some(unsigned) = found.as_u64() {
                return i64::try_from(unsigned).map_err(|_| {
                    ProviderError::invalid(format!("`{path}` is larger than this engine takes"))
                });
            }
            Err(ProviderError::invalid(format!(
                "`{path}` must be a whole number; it is {found}"
            )))
        }
        Some(other) => Err(ProviderError::invalid(format!(
            "`{path}` must be a whole number; it is {}",
            kind(other)
        ))),
        None => Err(ProviderError::invalid(format!("`{path}` is required"))),
    }
}

fn narrow_of<T>(found: Option<&Value>, path: &str) -> Result<T, ProviderError>
where
    T: TryFrom<i64>,
{
    let whole = integer_of(found, path)?;
    T::try_from(whole).map_err(|_| {
        ProviderError::invalid(format!(
            "`{path}` is {whole}, which does not fit the {} this engine declares",
            core::any::type_name::<T>()
        ))
    })
}

fn object_of(found: Option<&Value>, path: String) -> Result<Within<'_>, ProviderError> {
    match found {
        Some(Value::Object(object)) => Ok(Within { object, path }),
        Some(other) => Err(ProviderError::invalid(format!(
            "`{path}` must be an object; it is {}",
            kind(other)
        ))),
        None => Err(ProviderError::invalid(format!("`{path}` is required"))),
    }
}

/// An argument that must be a string, as the C string the engine takes.
///
/// Returned rather than passed straight through, because the buffer has
/// to outlive the call and a temporary would not: the generated code
/// binds it and then lends its pointer.
///
/// # Errors
///
/// `Invalid` naming the argument when it is missing, is not a string, or
/// contains a NUL — which C has no way to carry and which would
/// otherwise hand the engine a silently shortened path.
pub(crate) fn text(args: &Map<String, Value>, name: &str) -> Result<CString, ProviderError> {
    text_of(args.get(name), name)
}

fn text_of(found: Option<&Value>, path: &str) -> Result<CString, ProviderError> {
    match found {
        Some(Value::String(found)) => CString::new(found.as_str()).map_err(|_| {
            ProviderError::invalid(format!("`{path}` contains a NUL, which C cannot carry"))
        }),
        Some(other) => Err(ProviderError::invalid(format!(
            "`{path}` must be a string; it is {}",
            kind(other)
        ))),
        None => Err(ProviderError::invalid(format!("`{path}` is required"))),
    }
}

/// A field that must be present and may be null: a pointer, which C
/// has no absent state for and a null for.
///
/// `None` for null. Present-and-null is the only way to say "no pointer"
/// here, because every field is required and a key left out is a
/// mistake the caller is told about by name.
fn nullable_of<'a>(
    found: Option<&'a Value>,
    path: &str,
) -> Result<Option<&'a Value>, ProviderError> {
    match found {
        None => Err(ProviderError::invalid(format!(
            "`{path}` is required; pass null for none"
        ))),
        Some(Value::Null) => Ok(None),
        Some(value) => Ok(Some(value)),
    }
}

/// A string the engine lent: copied at once, because what it points at
/// belongs to the engine and may not outlive the next call.
///
/// `None` for null, which is how the engine says "no such thing" from a
/// function that returns a name.
#[allow(
    unsafe_code,
    reason = "reading a string the engine returned; the generated dispatch \
              is the only caller and hands it exactly what the engine gave"
)]
pub(crate) fn borrowed(pointer: *const c_char) -> Option<String> {
    if pointer.is_null() {
        return None;
    }
    // SAFETY: the caller passes a pointer the engine returned from a
    // function declared to return a NUL-terminated string.
    Some(
        unsafe { CStr::from_ptr(pointer) }
            .to_string_lossy()
            .into_owned(),
    )
}

/// A string the engine fills a buffer with, by the engine's own
/// documented protocol: the call answers **the length it wanted**, and a
/// buffer too short is filled to its limit rather than refused.
///
/// So the first call is made into a buffer generous enough for every
/// answer these functions have — which is one call, no allocation, and
/// no probe — and only an answer that did not fit is asked for again
/// into a buffer sized to what it said it wanted. `call` is invoked at
/// most twice and must be the same call both times.
///
/// The `size_t` a function like this returns is therefore **the
/// protocol's bookkeeping and not an answer**: the string it describes
/// has already arrived in full, which is why the generated dispatch
/// reports the string alone.
///
/// # Errors
///
/// `Refused` when the second call wanted more than the first — a
/// provider whose answer changed underneath the fill, which a caller
/// must be told about rather than handed a truncation.
pub(crate) fn fill<F>(function: &str, mut call: F) -> Result<String, ProviderError>
where
    F: FnMut(*mut c_char, usize) -> usize,
{
    /// Room for every string these functions answer with, a body name and
    /// a formatted angle among them, on the stack and in one call.
    ///
    /// Not a limit: an answer longer than this is asked for again into
    /// exactly the room it wanted. It is the point at which one call
    /// stops being enough, and it is generous because the engine's own
    /// fast path for a formatted angle needs sixty-four bytes to avoid a
    /// copy.
    const ROOMY: usize = 128;

    let mut buffer = [0_u8; ROOMY];
    let wanted = call(buffer.as_mut_ptr().cast(), buffer.len());
    if let Some(answer) = fitted(&buffer, wanted) {
        return Ok(answer);
    }
    let mut grown = vec![0_u8; wanted.saturating_add(1)];
    let again = call(grown.as_mut_ptr().cast(), grown.len());
    fitted(&grown, again).ok_or_else(|| ProviderError::Refused {
        detail: format!(
            "`{function}` wanted {wanted} bytes and then {again}; its answer changed between \
             the two calls"
        ),
    })
}

/// What the engine wrote, if what it wanted fit in what it was given.
///
/// `wanted == len` is a truncation and not a fit: the engine keeps a byte
/// of the buffer for the NUL, so a buffer exactly as long as the answer
/// came back one character short.
fn fitted(buffer: &[u8], wanted: usize) -> Option<String> {
    if wanted >= buffer.len() {
        return None;
    }
    buffer
        .get(..wanted)
        .map(|written| String::from_utf8_lossy(written).into_owned())
}

/// The engine's own status, as the port's error, for a call made without
/// a context — which therefore has no record to read a message from.
///
/// # Errors
///
/// Anything but success, carrying the engine's numeric code so a caller
/// can match on it, and the function's name so they know which call it
/// came from — the pass-through calls by name, so a bare code would leave
/// them guessing.
pub(crate) fn status(code: teimeris::sys::tm_status, function: &str) -> Result<(), ProviderError> {
    if code == 0 {
        return Ok(());
    }
    Err(refusal(code, function, Said::NoContext))
}

/// What the context's error record said **before** a call: nothing when
/// it was clean, and a copy when it held an earlier failure.
///
/// A copy is needed because the record is not cleared by every call, and
/// a failure that returns without writing it leaves the previous one's
/// message in place (`05-testing/02-engine-findings.md` D3). Comparing
/// against what was there before is the only way to attach a message to
/// the call that wrote it and never to one that did not.
pub(crate) struct Record(Option<Recorded>);

/// One failure as the engine recorded it.
#[derive(PartialEq, Eq)]
struct Recorded {
    status: teimeris::sys::tm_status,
    detail: i32,
    message: Option<String>,
    context: Option<String>,
}

/// The context's error record as it stands.
///
/// Costs a copy only while the record holds a failure; a clean record,
/// which is what every successful call leaves, costs two reads.
#[allow(
    unsafe_code,
    reason = "reading the error record of the adapter's own context, which the engine \
              documents as never null and valid until the next call"
)]
pub(crate) fn record(context: *const teimeris::sys::tm_context) -> Record {
    // SAFETY: the context is the adapter's own and live; `tm_last_error`
    // never returns null for one, and what it points at is read and
    // copied before any other call is made.
    let recorded = unsafe { &*teimeris::sys::tm_last_error(context) };
    if recorded.status == 0 {
        return Record(None);
    }
    Record(Some(Recorded {
        status: recorded.status,
        detail: recorded.detail,
        message: borrowed(recorded.message),
        context: borrowed(recorded.context),
    }))
}

/// The engine's own status, as the port's error, with the engine's own
/// message when **this** call wrote one.
///
/// # Errors
///
/// Anything but success: the engine's numeric code, the function's name,
/// the status's name, and the message and context the engine recorded
/// for the failure — `the output holds 1 of 2 julian days` — when the
/// record changed across the call. When it did not, the refusal says the
/// engine recorded nothing, rather than repeating an earlier failure's
/// words as this one's.
pub(crate) fn checked(
    context: *const teimeris::sys::tm_context,
    before: &Record,
    code: teimeris::sys::tm_status,
    function: &str,
) -> Result<(), ProviderError> {
    if code == 0 {
        return Ok(());
    }
    let after = record(context);
    let said = match (after.0, &before.0) {
        (Some(after), Some(before)) if after == *before => Said::Unrecorded,
        (Some(after), _) => Said::Recorded(after),
        (None, _) => Said::Unrecorded,
    };
    Err(refusal(code, function, said))
}

/// What the engine is known to have said about one failure.
enum Said {
    /// The call had no context, so there is no record to read.
    NoContext,
    /// The record did not change across the call: whatever it holds is an
    /// earlier failure's.
    Unrecorded,
    /// The call wrote this.
    Recorded(Recorded),
}

/// One refusal's words: the function, the code and its name, and what the
/// engine recorded if it is known to be this call's.
#[allow(
    unsafe_code,
    reason = "naming a status through the engine's own table of static names"
)]
fn refusal(code: teimeris::sys::tm_status, function: &str, said: Said) -> ProviderError {
    // SAFETY: `tm_status_name` takes any integer and answers a static
    // string, or null for a code it does not know.
    let name = borrowed(unsafe { teimeris::sys::tm_status_name(code) })
        .unwrap_or_else(|| "an unnamed status".to_owned());
    let said = match said {
        Said::NoContext => String::new(),
        Said::Unrecorded => "; the engine recorded no message for it".to_owned(),
        Said::Recorded(recorded) => {
            let message = recorded.message.unwrap_or_default();
            match recorded.context {
                Some(context) if !context.is_empty() => format!(": {message} ({context})"),
                _ => format!(": {message}"),
            }
        }
    };
    ProviderError::Provider {
        code: crate::CODE_BASE + code,
        detail: format!("`{function}` refused with status {code}, {name}{said}"),
    }
}

/// A status no engine function writes, put in every element of a batch's
/// output before the call so the call can be told to have written each
/// one.
///
/// The engine's statuses are zero and a handful of small negatives; the
/// most negative `int` is none of them, and a test holds that.
pub(crate) const UNWRITTEN: teimeris::sys::tm_status = teimeris::sys::tm_status::MIN;

/// Marks every element of a batch's room as not yet written.
pub(crate) fn unwritten<T>(
    room: &mut [T],
    status: impl Fn(&mut T) -> &mut teimeris::sys::tm_status,
) {
    for one in room {
        *status(one) = UNWRITTEN;
    }
}

/// A batch's refusal, **forgiven** when the engine wrote every element; and
/// on success, every element's status made true.
///
/// A batch whose elements each carry their own status returns the first
/// failure as its own but keeps computing, and refusing the whole answer
/// would throw away every element that succeeded. The marks tell the two
/// failures apart: one refused before or while filling leaves elements
/// still marked, and stands; one that filled every element, each recording
/// its own status, answers them.
///
/// On success an element still marked is one the engine did not need to
/// give a status — a rotation copies a position rather than computing it —
/// and a call that succeeded succeeded for it, so it is `TM_OK` and the
/// mark never reaches a caller.
///
/// # Errors
///
/// `result`, when it is an error and any element is still marked or there
/// are none.
pub(crate) fn settled<T>(
    result: Result<(), ProviderError>,
    elements: &mut [T],
    status: impl Fn(&mut T) -> &mut teimeris::sys::tm_status,
) -> Result<(), ProviderError> {
    match result {
        Ok(()) => {
            for one in elements.iter_mut() {
                let recorded = status(one);
                if *recorded == UNWRITTEN {
                    *recorded = 0;
                }
            }
            Ok(())
        }
        Err(_)
            if !elements.is_empty() && elements.iter_mut().all(|one| *status(one) != UNWRITTEN) =>
        {
            Ok(())
        }
        refused => refused,
    }
}

/// What a JSON value is, for a message that has to say so.
fn kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        reason = "a test fails by panicking"
    )]

    use super::{
        MOST_BYTES, borrowed, extent, fill, gather, integer, narrow, number, numbers, object,
        objects, optional_object, room, text, wholes,
    };
    use core::ffi::c_char;
    use serde_json::json;

    fn args(value: &serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
        value.as_object().expect("an object").clone()
    }

    #[test]
    fn a_number_is_read_and_a_missing_one_is_named() {
        let map = args(&json!({ "jd": 2_451_545.5 }));
        assert!((number(&map, "jd").unwrap() - 2_451_545.5).abs() < 1e-9);
        let error = number(&map, "jd_ut1").unwrap_err();
        assert!(
            error.to_string().contains("jd_ut1"),
            "the refusal must name the argument: {error}"
        );
    }

    /// A fractional value for a whole-number parameter is refused rather
    /// than truncated: the engine's integers are counts, ids and enum
    /// members, and 2.7 becoming 2 answers a question nobody asked.
    #[test]
    fn a_fraction_is_refused_where_a_whole_number_is_wanted() {
        let map = args(&json!({ "system": 2.7, "count": 3, "flag": true }));
        let error = integer(&map, "system").unwrap_err();
        assert!(error.to_string().contains("whole"), "{error}");
        assert_eq!(integer(&map, "count").unwrap(), 3);
        assert_eq!(
            integer(&map, "flag").unwrap(),
            1,
            "a JSON boolean is a C flag"
        );
    }

    /// A struct's field is refused by its whole path, because the field's
    /// own name alone — `year` — would not say which of a call's structs
    /// it belongs to.
    #[test]
    fn a_field_is_named_by_its_whole_path() {
        let map = args(&json!({
            "local": { "year": 2026, "month": 9.5, "at": { "lon": "east" } },
            "flat": 3,
        }));
        let local = object(&map, "local").unwrap();
        assert_eq!(local.narrow::<i32>("year").unwrap(), 2026);
        let month = local.narrow::<i32>("month").unwrap_err();
        assert!(month.to_string().contains("`local.month`"), "{month}");
        let missing = local.number("day").unwrap_err();
        assert!(
            missing.to_string().contains("`local.day` is required"),
            "{missing}"
        );
        let deep = local.nested("at").unwrap().number("lon").unwrap_err();
        assert!(deep.to_string().contains("`local.at.lon`"), "{deep}");
        let flat = object(&map, "flat")
            .err()
            .expect("a number is not an object");
        assert!(flat.to_string().contains("must be an object"), "{flat}");
        let wide = narrow::<u8>(&map, "flat").unwrap();
        assert_eq!(wide, 3);
    }

    /// Left out and null are both "no struct"; a value of the wrong kind
    /// is a mistake and is said to be one.
    #[test]
    fn an_optional_struct_is_absent_or_null_and_nothing_else() {
        let map = args(&json!({ "observer": null, "eye": 4, "atm": {} }));
        assert!(optional_object(&map, "missing").unwrap().is_none());
        assert!(optional_object(&map, "observer").unwrap().is_none());
        assert!(optional_object(&map, "atm").unwrap().is_some());
        let error = optional_object(&map, "eye")
            .err()
            .expect("a number is not a struct");
        assert!(
            error.to_string().contains("`eye` must be an object"),
            "{error}"
        );
    }

    /// The wrong kind is named as what it is, because a caller writing
    /// against a manifest has usually passed the right name and the wrong
    /// shape.
    #[test]
    fn the_wrong_kind_is_named() {
        let map = args(&json!({ "jd": "2451545.5" }));
        let error = number(&map, "jd").unwrap_err();
        assert!(error.to_string().contains("a string"), "{error}");
    }

    /// A NUL inside a string is refused rather than carried, because C
    /// stops there: a path with one would reach the engine silently
    /// shortened, and the engine would answer about a different file.
    #[test]
    fn a_nul_inside_a_string_is_refused() {
        let map = args(&json!({ "path": "de441.eph", "cut": "de441\u{0}.eph" }));
        assert_eq!(text(&map, "path").unwrap().to_str().unwrap(), "de441.eph");
        let error = text(&map, "cut").unwrap_err();
        assert!(error.to_string().contains("cut"), "{error}");
        assert!(error.to_string().contains("NUL"), "{error}");
    }

    /// The engine says "there is no such name" by answering null, and
    /// JSON has a word for that which is not the empty string.
    #[test]
    fn a_null_string_is_none_and_not_empty() {
        assert_eq!(borrowed(core::ptr::null()), None);
        let word = c"Aries";
        assert_eq!(
            borrowed(word.as_ptr().cast::<c_char>()),
            Some("Aries".to_string())
        );
    }

    /// The fill protocol in all three of its outcomes, against a fake
    /// engine: this is the part the generator cannot test, because no
    /// function this adapter offers answers more than a line and the
    /// second call would never run against the real one.
    #[test]
    fn a_fill_asks_twice_only_when_the_first_answer_did_not_fit() {
        /// Writes `answer` into whatever room it is given, truncating to
        /// fit, and returns the length it wanted — the engine's own
        /// documented contract.
        fn engine(answer: &str, buffer: *mut c_char, capacity: usize) -> usize {
            if capacity > 0 {
                let room = capacity - 1;
                let take = room.min(answer.len());
                // SAFETY: the caller is `fill`, which passes a buffer of
                // the capacity it says; `take` is inside it.
                #[allow(unsafe_code, reason = "a fake engine, filling as C does")]
                unsafe {
                    core::ptr::copy_nonoverlapping(answer.as_ptr(), buffer.cast::<u8>(), take);
                    buffer.add(take).write(0);
                }
            }
            answer.len()
        }

        let mut calls = 0_u32;
        let short = fill("short", |buffer, capacity| {
            calls += 1;
            engine("Sun", buffer, capacity)
        })
        .unwrap();
        assert_eq!(short, "Sun");
        assert_eq!(calls, 1, "an answer that fits costs one call");

        let long = "x".repeat(400);
        let mut calls = 0_u32;
        let grown = fill("long", |buffer, capacity| {
            calls += 1;
            engine(&long, buffer, capacity)
        })
        .unwrap();
        assert_eq!(grown, long, "the whole answer, not the first bufferful");
        assert_eq!(calls, 2, "one to find out, one to fill");

        // A provider whose answer changed underneath the fill. Told
        // about, rather than papered over with a truncation.
        let mut calls = 0_u32;
        let Err(error) = fill("growing", |buffer, capacity| {
            calls += 1;
            engine(&"y".repeat(200 * calls as usize), buffer, capacity)
        }) else {
            panic!("an answer that keeps growing must be refused");
        };
        assert!(error.to_string().contains("growing"), "{error}");
    }

    /// An array is refused by the element that is wrong, because a batch
    /// of a thousand dates with one bad month is a needle otherwise.
    #[test]
    fn an_array_is_refused_by_the_element_that_is_wrong() {
        let map = args(&json!({
            "jds": [2_451_545.0, 2_451_546],
            "bodies": [0, 1, 4_294_967_296_i64],
            "dts": [{ "year": 2026 }, { "year": "2027" }],
            "flat": 3,
        }));
        assert_eq!(
            numbers(&map, "jds").unwrap(),
            vec![2_451_545.0, 2_451_546.0]
        );
        let wide = wholes::<i32>(&map, "bodies").unwrap_err();
        assert!(wide.to_string().contains("`bodies[2]`"), "{wide}");
        let read = |within: &super::Within<'_>| within.narrow::<i32>("year");
        let bad = objects(&map, "dts", read).unwrap_err();
        assert!(bad.to_string().contains("`dts[1].year`"), "{bad}");
        let flat = numbers(&map, "flat").unwrap_err();
        assert!(flat.to_string().contains("must be an array"), "{flat}");
        let missing = numbers(&map, "none").unwrap_err();
        assert!(
            missing.to_string().contains("`none` is required"),
            "{missing}"
        );
        assert!(
            numbers(&args(&json!({ "empty": [] })), "empty")
                .unwrap()
                .is_empty()
        );
    }

    /// Room is refused past the bound and by the output's name, rather
    /// than an allocation the caller sized aborting the process.
    #[test]
    fn room_is_bounded_and_its_product_checked() {
        let small: Vec<f64> = room(3, "out").unwrap();
        assert_eq!(small, vec![0.0; 3]);
        let huge = room::<f64>(MOST_BYTES, "out").unwrap_err();
        assert!(huge.to_string().contains("`out`"), "{huge}");
        assert_eq!(extent(&[3, 4], "out").unwrap(), 12);
        assert_eq!(extent(&[], "out").unwrap(), 1);
        let overflow = extent(&[usize::MAX, 2], "out").unwrap_err();
        assert!(overflow.to_string().contains("no buffer"), "{overflow}");
    }

    /// The gather protocol in all three of its outcomes, against a fake
    /// engine that writes what fits and reports how many there are.
    #[test]
    fn a_gather_asks_twice_only_when_there_were_more() {
        fn engine(there: usize, room: &mut [u32]) -> usize {
            for (at, slot) in room.iter_mut().take(there).enumerate() {
                *slot = u32::try_from(at).unwrap();
            }
            there
        }

        let mut calls = 0;
        let few = gather("few", |room| {
            calls += 1;
            Ok(engine(3, room))
        })
        .unwrap();
        assert_eq!((few, calls), (vec![0, 1, 2], 1));

        let mut calls = 0;
        let many = gather("many", |room| {
            calls += 1;
            Ok(engine(100, room))
        })
        .unwrap();
        assert_eq!(
            (many.len(), many.last().copied(), calls),
            (100, Some(99), 2)
        );

        let mut calls = 0_usize;
        let Err(error) = gather::<u32>("growing", |_| {
            calls += 1;
            Ok(100 * calls)
        }) else {
            panic!("an answer that keeps growing must be refused");
        };
        assert!(error.to_string().contains("growing"), "{error}");
    }

    /// What `Keep` hands out is readable for as long as it lives, however
    /// many things were kept after it, and a pointer field is refused by
    /// its path unless it is present — null or not.
    #[test]
    fn a_kept_string_and_struct_outlive_what_is_kept_after_them() {
        let keep = super::Keep::default();
        let first = keep.text(std::ffi::CString::new("Regulus").unwrap());
        let value = keep.value([1.5_f64, 2.5]);
        for at in 0..1000 {
            keep.text(std::ffi::CString::new(format!("star {at}")).unwrap());
            keep.value(at);
        }
        assert_eq!(borrowed(first).as_deref(), Some("Regulus"));
        #[allow(unsafe_code, reason = "reading back what `Keep` holds")]
        // SAFETY: `keep` is alive and holds the array `value` points at.
        let read = unsafe { *value };
        assert_eq!(
            read.map(f64::to_bits),
            [1.5_f64.to_bits(), 2.5_f64.to_bits()]
        );

        let map =
            args(&json!({ "req": { "star": null, "observer": { "lat": 1.0 }, "cut": "a\u{0}" } }));
        let req = object(&map, "req").unwrap();
        assert!(req.text("star", &keep).unwrap().is_null());
        let missing = req.text("name", &keep).unwrap_err();
        assert!(
            missing
                .to_string()
                .contains("`req.name` is required; pass null"),
            "{missing}"
        );
        let cut = req.text("cut", &keep).unwrap_err();
        assert!(
            cut.to_string().contains("`req.cut` contains a NUL"),
            "{cut}"
        );
        let observer = req
            .pointer("observer", &keep, |within| within.number("lat"))
            .unwrap();
        #[allow(unsafe_code, reason = "reading back what `Keep` holds")]
        // SAFETY: `keep` is alive and holds the number `observer` points at.
        let lat = unsafe { *observer };
        assert!((lat - 1.0).abs() < f64::EPSILON);
        let deep = req
            .pointer("observer", &keep, |within| within.number("lon"))
            .unwrap_err();
        assert!(deep.to_string().contains("`req.observer.lon`"), "{deep}");
    }

    /// A null pointer comes back as null, not as a struct of zeros.
    #[test]
    fn a_null_pointer_comes_back_null() {
        #[allow(
            clippy::trivially_copy_pass_by_ref,
            reason = "the shape `pointed` takes, which a generated writer has"
        )]
        fn write(value: &f64) -> serde_json::Value {
            json!(value)
        }
        assert_eq!(super::pointed(core::ptr::null::<f64>(), write), json!(null));
        let one = 4.5_f64;
        assert_eq!(super::pointed(&raw const one, write), json!(4.5));
    }

    /// A batch refusal stands while any element is unwritten, and is
    /// forgiven once every element carries a status of its own.
    #[test]
    fn a_batch_answers_when_every_element_was_written() {
        use super::{UNWRITTEN, settled, unwritten};
        use teistro_port_ephemeris::ProviderError;
        let refused =
            || -> Result<(), ProviderError> { Err(ProviderError::invalid("first failure")) };
        let mut room = [(0.0_f64, 0); 3];
        unwritten(&mut room, |one| &mut one.1);
        assert!(room.iter().all(|one| one.1 == UNWRITTEN));
        assert!(
            settled(refused(), &mut room, |one| &mut one.1).is_err(),
            "nothing was written"
        );
        room[0].1 = 0;
        room[1].1 = -2;
        assert!(
            settled(refused(), &mut room, |one| &mut one.1).is_err(),
            "one is unwritten"
        );
        room[2].1 = 0;
        assert!(
            settled(refused(), &mut room, |one| &mut one.1).is_ok(),
            "each has its own"
        );
        assert!(
            settled(refused(), &mut room[..0], |one| &mut one.1).is_err(),
            "none to answer"
        );

        // A success never hands a caller the mark.
        let mut copied = [(1.0_f64, 0); 2];
        unwritten(&mut copied, |one| &mut one.1);
        copied[1].1 = -3;
        assert!(settled(Ok(()), &mut copied, |one| &mut one.1).is_ok());
        assert_eq!((copied[0].1, copied[1].1), (0, -3));

        // No status the engine names is the mark.
        for status in -8..=0 {
            assert_ne!(status, UNWRITTEN);
        }
    }

    /// A length read from a field is refused when it is not one.
    #[test]
    fn a_field_that_is_not_a_length_is_refused() {
        use super::length;
        assert_eq!(length(12_usize, "req.day_count").unwrap(), 12);
        assert_eq!(length(7_i32, "req.system").unwrap(), 7);
        let negative = length(-1_i32, "req.system").unwrap_err();
        assert!(
            negative.to_string().contains("`req.system` is -1"),
            "{negative}"
        );
    }
}
