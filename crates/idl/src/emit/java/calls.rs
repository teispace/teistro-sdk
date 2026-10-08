//! `Calls`: one static method per entry point a binding calls after a
//! context is open, marshalled from the description the way every other
//! binding's is (`rules::results`, the parameter roles), and package-private:
//! the hand-written `Context` and `Teistro` call them, under the context's
//! lock, and give each the name and the shape a Java caller expects.
//!
//! A method takes the library and, for a context's call, the context's
//! handle; allocates what the call writes into in a confined arena; and
//! answers what the call hands back, or throws what the library said about
//! the refusal. A role with no rule here stops the generator, naming the
//! parameter, as the Python emitter's does.

use std::fmt::Write;

use super::values::{carrier, narrow, raw, shown, widen};
use super::{File, PACKAGE, identifier, javadoc, literal, preamble};
use crate::model::{Api, FunctionDef, ParamDef, Role, Scalar, StructRole, TypeRef};
use crate::names::{binding_type_name, camel, method_name, pascal};
use crate::rules::{
    Handed, constructor, destructor, factories, has_handshake, is_free_function, last_error,
    methods, pointee_struct, results, returned_scalar, returns_status,
};

/// The `Calls` file, and a record per call that hands back more than one thing.
pub(super) fn files(api: &Api, dir: &str) -> Vec<File> {
    let mut files = vec![File {
        path: format!("{dir}/Calls.java"),
        text: render(api),
    }];
    for (f, _) in calls(api) {
        let handed = results(api, f);
        if handed.len() > 1 {
            files.push(File {
                path: format!("{dir}/{}.java", result_record(api, f)),
                text: render_result(api, f, &handed),
            });
        }
    }
    files
}

/// The calls rendered: a context's methods, then the library's own.
fn calls(api: &Api) -> Vec<(&FunctionDef, bool)> {
    let mut out = Vec::new();
    for opaque in &api.opaques {
        let skipped: Vec<&str> = constructor(api, opaque)
            .into_iter()
            .chain(factories(api, opaque))
            .chain(destructor(api, opaque))
            .chain(last_error(api, opaque))
            .map(|f| f.name.as_str())
            .collect();
        out.extend(
            methods(api, opaque)
                .into_iter()
                .filter(|f| !skipped.contains(&f.name.as_str()))
                .map(|f| (f, true)),
        );
    }
    // The library's own calls, less what `Teistro` reads by hand (those
    // with no parameter: the versions, the default profile, the build)
    // and the frees and the status message, which `Calls` uses itself.
    out.extend(
        api.functions
            .iter()
            .filter(|f| {
                is_free_function(f)
                    && !f.params.is_empty()
                    && !f.name.ends_with("_status_message")
                    && !f.params.iter().any(|p| {
                        matches!(p.role, Role::StringFree | Role::BlobFree | Role::ErrorFree)
                    })
            })
            .map(|f| (f, false)),
    );
    out
}

/// A call's method name: a context's without its handle's prefix
/// (`calendarFromFixed`), the library's without the symbol prefix.
fn name(api: &Api, f: &FunctionDef) -> String {
    let bare = api
        .opaques
        .iter()
        .find(|o| methods(api, o).iter().any(|m| m.name == f.name))
        .map_or_else(
            || {
                f.name
                    .strip_prefix(&api.prefix)
                    .unwrap_or(&f.name)
                    .to_string()
            },
            |o| method_name(&api.prefix, &o.name, &f.name),
        );
    identifier(&bare)
}

/// The record a call that hands back more than one thing answers.
fn result_record(api: &Api, f: &FunctionDef) -> String {
    format!("{}Result", pascal(&name(api, f)))
}

/// The Java type of a by-value parameter.
fn value_type(p: &ParamDef) -> String {
    if let Some(name) = &p.meta.enum_name {
        return binding_type_name(name);
    }
    match &p.ty {
        TypeRef::Enum { name } => binding_type_name(name),
        TypeRef::Scalar { scalar } => carrier(*scalar).to_string(),
        _ => String::from("long"),
    }
}

/// The scalar a parameter or a pointee is.
fn scalar_of(api: &Api, ty: &TypeRef) -> Scalar {
    match ty {
        TypeRef::Scalar { scalar } => *scalar,
        TypeRef::Enum { name } => api.enum_named(name).map_or(Scalar::I32, |e| e.repr),
        TypeRef::Pointer { to, .. } => scalar_of(api, to),
        _ => Scalar::I32,
    }
}

/// The raw type a value is passed or returned as through a method handle.
fn raw_of(api: &Api, ty: &TypeRef) -> &'static str {
    match ty {
        TypeRef::Scalar { .. } | TypeRef::Enum { .. } => raw(scalar_of(api, ty)),
        _ => "MemorySegment",
    }
}

/// The FFM value layout a scalar is read through.
const fn value_layout(scalar: Scalar) -> &'static str {
    match scalar {
        Scalar::U8 | Scalar::I8 => "JAVA_BYTE",
        Scalar::U16 | Scalar::I16 => "JAVA_SHORT",
        Scalar::U32 | Scalar::I32 => "JAVA_INT",
        Scalar::U64 | Scalar::I64 | Scalar::Usize | Scalar::Isize => "JAVA_LONG",
        Scalar::F32 => "JAVA_FLOAT",
        Scalar::F64 => "JAVA_DOUBLE",
        Scalar::Bool => "JAVA_BOOLEAN",
    }
}

/// A role the description has no instance of in a call `Calls` renders.
#[expect(
    clippy::panic,
    reason = "a generator that cannot place a parameter stops the build rather than emitting a wrong marshalling"
)]
fn unplaced(role: &str, param: &str, function: &str) -> ! {
    panic!(
        "no Java rule for the `{role}` parameter `{param}` of `{function}`: the \
         description had no instance of that role in a call when the emitter was written \
         (03-design/binding-surface-measured.md \u{a7}1)"
    )
}

/// The Java type of one thing a call hands back.
fn handed_type(api: &Api, hand: &Handed<'_>) -> String {
    match hand {
        Handed::Returned(scalar) => carrier(*scalar).to_string(),
        Handed::Struct(p) => match pointee_struct(api, p) {
            Some(s) if shown(api, s) => binding_type_name(&s.name),
            _ => unplaced("struct_out", &p.name, "a call"),
        },
        Handed::Blob(_) => String::from("byte[]"),
        Handed::Owned(_) | Handed::Lent(_) => String::from("String"),
        Handed::Scalar(p) => carrier(scalar_of(api, &p.ty)).to_string(),
    }
}

/// The local a handed-back value is held in before it is read.
fn out_local(p: &ParamDef) -> String {
    identifier(&p.name)
}

/// How one handed-back value is read, once the call has answered.
fn handed_read(api: &Api, hand: &Handed<'_>) -> String {
    match hand {
        Handed::Returned(scalar) => widen(*scalar, "value"),
        Handed::Struct(p) => format!("{}.of({})", handed_type(api, hand), out_local(p)),
        Handed::Blob(p) => format!("taken(lib, {})", out_local(p)),
        Handed::Owned(p) => format!("owned(lib, {})", out_local(p)),
        Handed::Lent(p) => format!("Boundary.borrowed({})", out_local(p)),
        Handed::Scalar(p) => {
            let scalar = scalar_of(api, &p.ty);
            widen(
                scalar,
                &format!("{}.get({}, 0L)", out_local(p), value_layout(scalar)),
            )
        }
    }
}

/// What marshalling a call's parameters comes to: the Java parameters it
/// takes, the lines that set its arguments up, and the arguments in order.
struct Marshalling {
    signature: Vec<String>,
    setup: Vec<String>,
    args: Vec<String>,
}

/// One parameter's part, by its role: what the caller passes in, or what
/// the call writes into.
fn parameter(api: &Api, f: &FunctionDef, p: &ParamDef, index: usize, m: &mut Marshalling) {
    let local = identifier(&p.name);
    let label = literal(&camel(&p.name));
    match p.role {
        Role::StructOut | Role::BlobOut | Role::StringOut | Role::StrOut | Role::ScalarOut => {
            written(api, f, p, &local, m);
        }
        Role::Handle => {
            m.signature.push(String::from("MemorySegment context"));
            m.args.push(String::from("context"));
        }
        Role::Value => {
            let ty = value_type(p);
            m.signature.push(format!("{ty} {local}"));
            let scalar = scalar_of(api, &p.ty);
            let carried = if ty.chars().next().is_some_and(char::is_uppercase) {
                format!("Values.id({label}, {local})")
            } else {
                local.clone()
            };
            let from = if carried == local { ty.as_str() } else { "int" };
            let raw_value = narrow(scalar, from, &label, &carried);
            if raw_value == local {
                m.args.push(local);
            } else {
                m.setup
                    .push(format!("{} {local}Raw = {raw_value};", raw(scalar)));
                m.args.push(format!("{local}Raw"));
            }
        }
        Role::StringIn => {
            m.signature.push(format!("String {local}"));
            let text = if p.meta.nullable {
                local.clone()
            } else {
                format!("Objects.requireNonNull({local}, {label})")
            };
            m.setup.push(format!(
                "MemorySegment {local}Raw = Boundary.cString(arena, {text});"
            ));
            m.args.push(format!("{local}Raw"));
        }
        Role::BytesIn => {
            m.signature.push(format!("byte[] {local}"));
            m.setup.push(format!(
                "MemorySegment {local}Raw = arena.allocateFrom(JAVA_BYTE, Objects.requireNonNull({local}, {label}));"
            ));
            m.args.push(format!("{local}Raw"));
        }
        Role::Length => {
            let of = f
                .params
                .get(index.wrapping_sub(1))
                .map_or_else(String::new, |before| identifier(&before.name));
            m.args.push(format!("(long) {of}.length"));
        }
        Role::StructIn => {
            let ty = match pointee_struct(api, p) {
                Some(s) if shown(api, s) => binding_type_name(&s.name),
                _ => unplaced("struct_in", &p.name, &f.name),
            };
            m.signature.push(format!("{ty} {local}"));
            m.setup.push(format!(
                "MemorySegment {local}Raw = Objects.requireNonNull({local}, {label}).toC(arena);"
            ));
            m.args.push(format!("{local}Raw"));
        }
        Role::HandleOut
        | Role::VtableIn
        | Role::UserData
        | Role::StringFree
        | Role::BlobFree
        | Role::ErrorFree
        | Role::ArrayIn => unplaced(&format!("{:?}", p.role), &p.name, &f.name),
    }
}

/// A parameter the call writes into, allocated for it in the call's arena.
fn written(api: &Api, f: &FunctionDef, p: &ParamDef, local: &str, m: &mut Marshalling) {
    let local = local.to_string();
    match p.role {
        Role::StructOut => {
            let Some(s) = pointee_struct(api, p) else {
                unplaced("struct_out", &p.name, &f.name)
            };
            m.setup.push(format!(
                "MemorySegment {local} = arena.allocate(Native.{}.LAYOUT);",
                s.name
            ));
            // The library checks the handshake before it writes, and
            // refuses a size that is not its own.
            if has_handshake(s) {
                m.setup.push(format!(
                    "Native.{0}.STRUCT_SIZE.set({local}, 0L, (int) Native.{0}.SIZE);",
                    s.name
                ));
            }
            m.args.push(local);
        }
        Role::BlobOut | Role::StringOut | Role::StrOut => {
            let Some(s) = pointee_struct(api, p) else {
                unplaced("an out string", &p.name, &f.name)
            };
            m.setup.push(format!(
                "MemorySegment {local} = arena.allocate(Native.{}.LAYOUT);",
                s.name
            ));
            m.args.push(local);
        }
        Role::ScalarOut => {
            let scalar = scalar_of(api, &p.ty);
            m.setup.push(format!(
                "MemorySegment {local} = arena.allocate({});",
                value_layout(scalar)
            ));
            m.args.push(local);
        }
        _ => unplaced(&format!("{:?}", p.role), &p.name, &f.name),
    }
}

/// One call's method.
fn render_call(out: &mut String, api: &Api, f: &FunctionDef, on_context: bool) {
    let handed = results(api, f);
    let mut m = Marshalling {
        signature: vec![String::from("Native lib")],
        setup: Vec::new(),
        args: Vec::new(),
    };
    for (index, p) in f.params.iter().enumerate() {
        parameter(api, f, p, index, &mut m);
    }
    let Marshalling {
        signature,
        setup,
        args,
    } = m;
    let returns = match handed.as_slice() {
        [] => String::from("void"),
        [one] => handed_type(api, one),
        _ => result_record(api, f),
    };
    let method = name(api, f);
    out.push_str(&javadoc(&format!("`{}`: {}", f.name, f.doc), "    "));
    let mut body = String::new();
    for line in &setup {
        let _ = writeln!(body, "            {line}");
    }
    let invoke = format!("lib.{}.invokeExact({})", f.name, args.join(", "));
    if returns_status(api, f) {
        let status = raw_of(api, f.returns.as_ref().unwrap_or(&TypeRef::Void));
        let refusal = if on_context {
            "refused(lib, context, status)"
        } else {
            "refused(lib, status)"
        };
        let _ = writeln!(
            body,
            "            {status} status = Boundary.call(() -> ({status}) {invoke});\n            if (status != 0) {{\n                throw {refusal};\n            }}"
        );
    } else if let Some(scalar) = returned_scalar(f) {
        let _ = writeln!(
            body,
            "            {0} value = Boundary.call(() -> ({0}) {invoke});",
            raw(scalar)
        );
    } else {
        let _ = writeln!(
            body,
            "            Boundary.call(() -> {{\n                {invoke};\n                return null;\n            }});"
        );
    }
    match handed.as_slice() {
        [] => {}
        [one] => {
            let _ = writeln!(body, "            return {};", handed_read(api, one));
        }
        many => {
            let _ = writeln!(
                body,
                "            return new {}(\n                    {});",
                result_record(api, f),
                many.iter()
                    .map(|h| handed_read(api, h))
                    .collect::<Vec<_>>()
                    .join(",\n                    ")
            );
        }
    }
    // An arena only where something is allocated: javac refuses a
    // resource its body never names.
    let arena = setup.iter().any(|line| line.contains("arena"));
    let _ = writeln!(
        out,
        "    static {returns} {method}({}) {{",
        signature.join(", ")
    );
    if arena {
        let _ = write!(
            out,
            "        try (Arena arena = Arena.ofConfined()) {{\n{body}        }}\n"
        );
    } else {
        for line in body.lines() {
            let _ = writeln!(out, "{}", line.strip_prefix("    ").unwrap_or(line));
        }
    }
    out.push_str("    }\n\n");
}

/// The record a call that hands back more than one thing answers, as its
/// own public file so the façade can hand it on.
fn render_result(api: &Api, f: &FunctionDef, handed: &[Handed<'_>]) -> String {
    let mut out = preamble(api, PACKAGE);
    let components: Vec<String> = handed
        .iter()
        .map(|h| format!("{} {}", handed_type(api, h), identifier(&h.name())))
        .collect();
    let mut doc = format!("What `{}` hands back.\n", f.name);
    for h in handed {
        let _ = write!(
            doc,
            "\n@param {} the call's `{}`",
            identifier(&h.name()),
            h.name()
        );
    }
    out.push_str(&javadoc(&doc, "").replace("&#64;param", "@param"));
    let _ = writeln!(
        out,
        "public record {}({}) {{}}",
        result_record(api, f),
        components.join(", ")
    );
    out
}

/// `Calls`, whole.
fn render(api: &Api) -> String {
    let mut out = preamble(api, PACKAGE);
    out.push_str("import static java.lang.foreign.ValueLayout.JAVA_BYTE;\nimport static java.lang.foreign.ValueLayout.JAVA_DOUBLE;\nimport static java.lang.foreign.ValueLayout.JAVA_INT;\nimport static java.lang.foreign.ValueLayout.JAVA_LONG;\nimport static java.lang.foreign.ValueLayout.JAVA_SHORT;\n\nimport java.lang.foreign.Arena;\nimport java.lang.foreign.MemorySegment;\nimport java.util.Objects;\n\nimport com.teispace.teistro.ffi.Native;\n\n");
    out.push_str("/**\n * Every entry point a binding calls once a context is open, marshalled\n * from the description: a method takes the library and, for a context's\n * call, its handle, and answers what the call hands back or throws what\n * the library said about the refusal. The caller holds the context's lock.\n */\n@SuppressWarnings(\"unused\")\nfinal class Calls {\n    private Calls() {}\n\n");
    out.push_str(&helpers(api));
    for (f, on_context) in calls(api) {
        render_call(&mut out, api, f, on_context);
    }
    out.push_str("}\n");
    out
}

/// The refusals and the takes every call shares, naming the entry points
/// the description gives them.
fn helpers(api: &Api) -> String {
    let find = |role: Role, fallback: &str| {
        api.functions
            .iter()
            .find(|f| f.params.iter().any(|p| p.role == role))
            .map_or_else(|| fallback.to_string(), |f| f.name.clone())
    };
    let string_free = find(Role::StringFree, "ts_string_free");
    let blob_free = find(Role::BlobFree, "ts_blob_free");
    let message = api
        .functions
        .iter()
        .find(|f| f.name.ends_with("_status_message"))
        .map_or_else(|| String::from("ts_status_message"), |f| f.name.clone());
    let reader = api
        .opaques
        .iter()
        .find_map(|o| last_error(api, o))
        .map_or_else(|| String::from("ts_context_last_error"), |f| f.name.clone());
    let error = api
        .structs
        .iter()
        .find(|s| s.role == StructRole::Error)
        .map_or_else(|| String::from("TsError"), |s| s.name.clone());
    let blob = api
        .functions
        .iter()
        .flat_map(|f| &f.params)
        .find(|p| p.role == Role::BlobFree)
        .and_then(|p| pointee_struct(api, p))
        .map_or_else(|| String::from("TsBlob"), |s| s.name.clone());
    format!(
        r#"    /**
     * The exception for a status a context's call answered, from the
     * context's own record of it, read before anything else touches the
     * context.
     */
    static TeistroException refused(Native lib, MemorySegment context, int status) {{
        try (Arena arena = Arena.ofConfined()) {{
            MemorySegment error = arena.allocate(Native.{error}.LAYOUT);
            Native.{error}.STRUCT_SIZE.set(error, 0L, (int) Native.{error}.SIZE);
            int read = Boundary.call(() -> (int) lib.{reader}.invokeExact(context, error));
            if (read != 0) {{
                return refused(lib, status);
            }}
            return Boundary.refusal(status, error);
        }}
    }}

    /** The exception for a status a call with no context answered: the status's own sentence. */
    static TeistroException refused(Native lib, int status) {{
        Status code = Status.of(status);
        MemorySegment message = Boundary.call(() -> (MemorySegment) lib.{message}.invokeExact(status));
        String text = Boundary.text(message);
        return new TeistroException(code, text.isEmpty() ? code.key() : text, "", "", "", "");
    }}

    /** A string the library allocated, copied out and freed. */
    static String owned(Native lib, MemorySegment string) {{
        try {{
            return Boundary.owned(string);
        }} finally {{
            Boundary.call(() -> {{
                lib.{string_free}.invokeExact(string);
                return null;
            }});
        }}
    }}

    /**
     * A result blob the library allocated, copied out and freed: one copy,
     * so nothing a caller holds reads the library's memory once it is let go.
     */
    @SuppressWarnings("restricted")
    static byte[] taken(Native lib, MemorySegment blob) {{
        try {{
            MemorySegment data = (MemorySegment) Native.{blob}.DATA.get(blob, 0L);
            long len = (long) Native.{blob}.LEN.get(blob, 0L);
            if (len == 0 || data.equals(MemorySegment.NULL)) {{
                return new byte[0];
            }}
            return data.reinterpret(len).toArray(JAVA_BYTE);
        }} finally {{
            Boundary.call(() -> {{
                lib.{blob_free}.invokeExact(blob);
                return null;
            }});
        }}
    }}

"#
    )
}
