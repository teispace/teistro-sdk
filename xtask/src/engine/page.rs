//! The measured page, from a classification already done.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use super::classify::{
    Blocker, Sizing, Standing, carries, carries_a_string, carries_a_struct, carries_an_array,
    describe, fills_a_string, has_role, mutates_engine_state, returns_a_string, sizing,
};
use super::idl::{Extent, Function, Idl};
use super::{DISPATCH, IDL};
use crate::measure::{count, spelled};

/// One function as the page prints it: the description it came from, so
/// every figure on the page is computed from the same reading, and the
/// reason the classifier gave.
pub(crate) struct Row<'a> {
    function: &'a Function,
    why: &'a str,
}

impl Row<'_> {
    fn name(&self) -> &str {
        &self.function.name
    }

    fn mutates(&self) -> bool {
        mutates_engine_state(&self.function.name)
    }
}

/// How many rows describe a function satisfying a predicate, as the
/// page's prose needs it.
///
/// Over the description rather than over the row, so the predicates are
/// the same ones the classifier used and no second definition of "takes
/// a string" can drift away from the first.
pub(crate) fn count_where(rows: &[Row<'_>], predicate: impl Fn(&Function) -> bool) -> usize {
    rows.iter().filter(|row| predicate(row.function)).count()
}

/// The measurement, from a reading already done.
pub(crate) fn page(idl: &Idl, classified: &[(&Function, Standing, &'static str)]) -> String {
    let mut by_standing: BTreeMap<Standing, Vec<Row<'_>>> = BTreeMap::new();
    for (function, standing, why) in classified {
        by_standing
            .entry(*standing)
            .or_default()
            .push(Row { function, why });
    }
    let callable = by_standing.get(&Standing::Callable).map_or(0, Vec::len);
    let owned = by_standing.get(&Standing::Owned).map_or(0, Vec::len);
    let unlearned = by_standing.get(&Standing::Unlearned).map_or(0, Vec::len);

    let mut out = String::new();
    let _ = writeln!(out, "# The engine passthrough, measured\n");
    let _ = writeln!(
        out,
        "Status: `generated` by `cargo xtask engine`, gated by `check-engine`. Do not edit. Read from `{IDL}`, the engine's own generated description, vendored beside the adapter at version `{}`. The same reading writes `{DISPATCH}`, so a figure here and the code that answers it cannot disagree.\n",
        idl.version
    );
    let _ = writeln!(
        out,
        "ADR-0030 puts an engine's own functions at `sdk.engine.*`, so that a consumer can reach what the SDK has not ported. The port says how — \"the marshalling belongs to the adapter, generated from the engine's manifest\" — and this is the measurement that generation is sized from.\n"
    );
    let _ = writeln!(
        out,
        "The engine describes **{} functions**, beside {} structs, {} enums and {} callbacks.\n",
        count(idl.functions.len()),
        count(idl.structs.len()),
        count(idl.enums.len()),
        count(idl.callbacks.len()),
    );

    let _ = writeln!(out, "| standing | functions | what it means |");
    let _ = writeln!(out, "|---|---:|---|");
    let _ = writeln!(
        out,
        "| **callable** | {callable} | offered at `sdk.engine.*` today |"
    );
    let _ = writeln!(
        out,
        "| **the adapter's own** | {owned} | never offered, whatever their shape |"
    );
    let _ = writeln!(
        out,
        "| **not yet marshalled** | {unlearned} | a queue for the generator, not a refusal |"
    );
    let _ = writeln!(out);

    out.push_str(&owned_section(by_standing.get(&Standing::Owned)));
    out.push_str(&callable_section(by_standing.get(&Standing::Callable)));
    out.push_str(&queue_section(by_standing.get(&Standing::Unlearned)));
    out.push_str(&facade_section(
        by_standing
            .get(&Standing::Callable)
            .map_or(&[][..], |rows| rows.as_slice()),
    ));
    out.push_str(&limits_section());
    out
}

/// What shape the typed façade hands an answer back in, which is a
/// measurement rather than a taste.
pub(crate) fn facade_section(rows: &[Row<'_>]) -> String {
    let mut out = String::new();
    if rows.is_empty() {
        return out;
    }
    let mut by_count: BTreeMap<usize, Vec<&str>> = BTreeMap::new();
    for row in rows {
        by_count
            .entry(describe(row.function).gives.len())
            .or_default()
            .push(row.name());
    }
    let one = by_count.get(&1).map_or(0, Vec::len);
    let none = by_count.get(&0).map_or(0, Vec::len);
    let many: Vec<&str> = by_count
        .iter()
        .filter(|(count, _)| **count > 1)
        .flat_map(|(_, names)| names.iter().copied())
        .collect();
    let listed = many
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ");

    let _ = writeln!(out, "## What the typed façade hands back\n");
    let _ = writeln!(
        out,
        "ADR-0030 puts a typed façade in the **adapter's** package, generated from this same reading so that it cannot type an argument the dispatch would refuse by name. What a method answers with is decided here, by counting:\n"
    );
    let _ = writeln!(out, "| values answered | functions | the façade's answer |");
    let _ = writeln!(out, "|---:|---:|---|");
    let _ = writeln!(
        out,
        "| 1 | {one} | **the value itself** — a number, a string, a struct |"
    );
    let _ = writeln!(out, "| 0 | {none} | nothing |");
    let _ = writeln!(
        out,
        "| more | {} | a record: an object, a Dart record, a `TypedDict` |",
        many.len()
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "**{one} of the {} answer with exactly one value**, so a façade that always returned an object would have made every one of them an indexing exercise for the sake of {}. Those {} get a record apiece — {listed} — which is {} types per target rather than {}.\n",
        rows.len(),
        many.len(),
        many.len(),
        many.len(),
        rows.len()
    );
    let _ = writeln!(
        out,
        "The same count settles a question every target would otherwise have raised. `return` — the key a function's own return value comes back under — **never appears beside another key**: every one of those {} is a status-returning function with out-parameters. So no record field is ever named `return`, and no target has to rename a keyword it could not spell.\n",
        many.len()
    );
    out
}

pub(crate) fn owned_section(rows: Option<&Vec<Row<'_>>>) -> String {
    let mut out = String::new();
    let Some(rows) = rows else { return out };
    let _ = writeln!(out, "## What the adapter will not hand over\n");
    let _ = writeln!(
        out,
        "**{} functions, excluded by what they touch rather than by what they cost.** Each opens, closes or rebinds the context the adapter is holding, or the data bound to it for that context's life. A consumer who called one through the passthrough would close the context every other call depends on, or change the files underneath it.\n",
        spelled(rows.len())
    );
    let _ = writeln!(
        out,
        "They are named here rather than left to the marshaller to fail on, because the reason is a **boundary** and not a difficulty — teaching the generator more shapes must never bring them in.\n"
    );
    for row in rows {
        let _ = writeln!(out, "- `{}`", row.name());
    }
    let _ = writeln!(out);
    out
}

pub(crate) fn callable_section(rows: Option<&Vec<Row<'_>>>) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## What is callable\n");
    let Some(rows) = rows else {
        let _ = writeln!(out, "Nothing yet.\n");
        return out;
    };
    let mutating = count_where(rows, |function| mutates_engine_state(&function.name));
    // Counted from the same rows the table below prints, because the
    // sentence is a claim about them: an arm that carries a string is a
    // different amount of generated code from one that does not, and a
    // paragraph asserting "scalars alone" outlived the truth of it once
    // already.
    let takes = count_where(rows, |function| has_role(function, "string_in"));
    let fills = count_where(rows, fills_a_string);
    let lends = count_where(rows, returns_a_string);
    let strings = count_where(rows, carries_a_string);
    let structs = count_where(rows, carries_a_struct);
    let both = count_where(rows, |function| {
        carries_a_string(function) && carries_a_struct(function)
    });
    let arrays = count_where(rows, carries_an_array);
    let scalar_only = count_where(rows, |function| {
        !carries_a_string(function) && !carries_a_struct(function) && !carries_an_array(function)
    });
    let _ = writeln!(
        out,
        "**{} functions**. {} of them take and answer scalars and enums alone, which is the shape a JSON object carries without a marshaller having to know anything else.\n",
        spelled(rows.len()),
        spelled(scalar_only),
    );
    let _ = writeln!(
        out,
        "{} carry a string: {} read one the caller passes, {} fill a buffer of the marshaller's, and {} answer with one the engine lends and the marshaller copies before anything else can move it.\n",
        spelled(strings),
        spelled(takes),
        spelled(fills),
        spelled(lends),
    );
    let _ = writeln!(
        out,
        "{} carry a struct, which crosses as a JSON object keyed by the engine's own field names, nested as the struct nests; {} of them carry a string as well. Every field is required going in, and the struct's own `struct_size` crosses in neither direction — the arm fills it, because the engine reads the struct only as far as it says (`03-design/engine-passthrough.md`).\n",
        spelled(structs),
        spelled(both),
    );
    let outputs = |wanted: fn(&Sizing<'_>) -> bool| {
        rows.iter()
            .flat_map(|row| {
                row.function
                    .params
                    .iter()
                    .filter(|param| param.role == "array_out")
                    .filter_map(|param| sizing(row.function, param))
            })
            .filter(|sized| wanted(sized))
            .count()
    };
    let inputs = outputs(|sized| matches!(sized, Sizing::Inputs(_)));
    let asked = outputs(|sized| matches!(sized, Sizing::Asked { .. }));
    let totals = outputs(|sized| matches!(sized, Sizing::Total { .. }));
    let called = outputs(|sized| matches!(sized, Sizing::Called { .. }));
    let _ = writeln!(
        out,
        "{} carry an array, which crosses as a JSON array of whatever its element crosses as. The engine's description says how long each output is, and the marshaller sizes it from that and nothing else: {} are as long as the inputs they answer or a field of the request says, {} are as long as another function answers — the house cusps, `tm_house_cusp_count()` of the requested system, asked before the call — {} are as long as the caller asks — how many eclipses to find, which is an argument — and cut to the count the engine gives, and {} are as long as the engine says there are, which the marshaller learns by asking and asks again when there are more than fitted. No caller passes a capacity for an answer whose length is already decided.\n",
        spelled(arrays),
        spelled(inputs),
        spelled(called),
        spelled(asked),
        spelled(totals),
    );
    let _ = writeln!(out, "| function | carries | changes engine state |");
    let _ = writeln!(out, "|---|---|---|");
    for row in rows {
        let _ = writeln!(
            out,
            "| `{}` | {} | {} |",
            row.name(),
            carries(row.function),
            if row.mutates() { "**yes**" } else { "" }
        );
    }
    let _ = writeln!(out);
    if mutating > 0 {
        let _ = writeln!(
            out,
            "### The {} that change engine state\n",
            spelled(mutating)
        );
        let _ = writeln!(
            out,
            "These are **offered rather than refused**, and the column above is why the distinction is drawn at all. Reaching what the SDK has not ported is the point of the namespace; but after one of these the engine is answering under settings the SDK's own provenance does not record, so a chart cast afterwards says it was computed one way and was computed another.\n"
        );
        let _ = writeln!(
            out,
            "A consumer who wants the change *recorded* has the settings for it (ADR-0013's override policy), and one who wants it anyway can have it and knows what it costs. What the SDK will not do is make the choice quietly on their behalf.\n"
        );
    }
    out
}

pub(crate) fn queue_section(rows: Option<&Vec<Row<'_>>>) -> String {
    let mut out = String::new();
    let Some(rows) = rows else { return out };
    let _ = writeln!(out, "## What the marshaller has not learned\n");
    let mut by_reason: BTreeMap<(usize, &str), Vec<&str>> = BTreeMap::new();
    for row in rows {
        // In rank order, easiest first, so the table reads as the order
        // of work; a reason that is not a blocker comes after them all.
        let rank = Blocker::ALL
            .iter()
            .position(|blocker| blocker.wording() == row.why)
            .unwrap_or(Blocker::ALL.len());
        by_reason
            .entry((rank, row.why))
            .or_default()
            .push(row.name());
    }
    let _ = writeln!(
        out,
        "**{} functions**, grouped by the hardest thing in the way and listed easiest first. This is a queue rather than a refusal: each group is one shape the generator has to learn, and learning one brings its whole group in at once.\n",
        spelled(rows.len())
    );
    let _ = writeln!(out, "| what it takes or returns | functions | examples |");
    let _ = writeln!(out, "|---|---:|---|");
    for ((_, why), names) in &by_reason {
        let examples: Vec<String> = names.iter().take(3).map(|n| format!("`{n}`")).collect();
        let _ = writeln!(out, "| {why} | {} | {} |", names.len(), examples.join(", "));
    }
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "The order of work, with what each step releases, is in `03-design/engine-passthrough.md` §6; the figures there were measured by the same classification as this table.\n"
    );
    out.push_str(&unsized_section(rows));
    out
}

/// Every output array the marshaller cannot size, with the engine's own
/// reason, so the queue's row says *why* and not only how many.
fn unsized_section(rows: &[Row<'_>]) -> String {
    let mut out = String::new();
    let unsizable: Vec<(&str, &str, String)> = rows
        .iter()
        .flat_map(|row| {
            row.function
                .params
                .iter()
                .filter(|param| param.role == "array_out" && sizing(row.function, param).is_none())
                .map(|param| {
                    let why = match &param.extent {
                        Some(Extent::Unstated { why }) => why.clone(),
                        Some(Extent::Length { of }) => format!("as long as `{of}`, which the marshaller cannot read before the call"),
                        Some(Extent::Product { of }) => format!("the product of `{}`, which the marshaller cannot all read before the call", of.join("` × `")),
                        Some(Extent::Call { function, of }) => format!("what `{function}({})` answers, which the marshaller cannot call before the call", of.join(", ")),
                        _ => "no extent the marshaller can read".to_string(),
                    };
                    (row.name(), param.name.as_str(), why)
                })
        })
        .collect();
    if unsizable.is_empty() {
        return out;
    }
    let _ = writeln!(out, "### The outputs it cannot size\n");
    let _ = writeln!(
        out,
        "**{} output arrays**, each with the engine's own account of what decides its length. None is guessed: an output sized wrongly is a truncated answer at best, and at worst a refusal the caller has no way to fix.\n",
        spelled(unsizable.len())
    );
    let _ = writeln!(out, "| function | output | its length is |");
    let _ = writeln!(out, "|---|---|---|");
    for (function, output, why) in unsizable {
        let _ = writeln!(out, "| `{function}` | `{output}` | {why} |");
    }
    let _ = writeln!(out);
    out
}

pub(crate) fn limits_section() -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## What this does not measure\n");
    let _ = writeln!(
        out,
        "**Whether a callable function answers correctly.** This reads a description and classifies shapes; it does not call anything. What the marshalling produces is tested against the engine where the adapter's own tests run, and a function's presence here is a claim about its *shape* alone.\n"
    );
    let _ = writeln!(
        out,
        "**Any engine but this one.** The classification is of one vendored description. Another engine that answers `native_manifest` reaches `sdk.engine.call` by the dynamic route with no generation at all, and gets no typed façade until someone generates one from its description.\n"
    );
    let _ = writeln!(
        out,
        "**Whether a consumer should use any of it.** A call through this namespace is to a named engine and does not survive changing it — which is why the namespace is called `engine` and not `ephemeris` (ADR-0030). What proves universal is promoted into the port, and then it is portable and this page is no longer where it lives.\n"
    );
    out
}
