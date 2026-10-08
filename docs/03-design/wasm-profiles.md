# wasm profiles: one module per profile

Status: `draft`, 2026-10-08. Track A item 5 of the completion plan
(`07-roadmap/00-roadmap.md`), and ADR-0005's "wasm ships per-profile
binaries". Measured before designed, because the profiles' worth rests on
how much of the module each crate takes, and nothing had measured it.

## What the module is made of

The `compact` module of 2026-10-08, built with the workspace's `wasm`
profile and read before its name section is removed, is 12.9 MB:
5.64 MB of code, 2.05 MB of data, and the name section. The code is
attributed to crates by each function's mangled name (Rust's v0 scheme
names each crate in the path). A generic function instantiated for a
type of the SDK's is counted to the first SDK crate its name carries,
since it is there because that crate is. The script is
`cargo xtask wasm-crates`.

| crate | code (bytes) | share |
|---|---:|---:|
| `teistro_core` | 1 089 357 | 19.3% |
| `teistro_rules` | 771 702 | 13.7% |
| `teistro_muhurta` | 376 563 | 6.7% |
| `teistro` (the façade) | 350 355 | 6.2% |
| `teistro_ffi` | 293 454 | 5.2% |
| `teistro_intl` | 274 295 | 4.9% |
| `teistro_panchanga` | 220 459 | 3.9% |
| `teistro_tajika` | 190 734 | 3.4% |
| `teistro_astro` | 189 953 | 3.4% |
| `teistro_western` | 164 189 | 2.9% |
| `teistro_hellenistic` | 157 411 | 2.8% |
| every other crate | 1 564 447 | 27.7% |

`teistro_core`'s code is many small functions over the catalogue's
generated enums and the settings' patch deserialisers, each a few
kilobytes; no one function stands out.

The data is mostly what crates embed: the shipped rule corpus and its
tables as JSON (956 kB), the document's JSON Schema (224 kB) and the
locale bundles (270 kB). Minifying the JSON would save 488 kB raw and
16 kB gzipped, under 1% of what a browser downloads, so it is not done
for its own sake.

## What a profile could save

The profiles leave out whole crates. Their code, from the table:

| profile | leaves out | code kept |
|---|---|---:|
| `full` | nothing | 100% |
| `baseline-parity` | the Western and Hellenistic crates | 94% |
| `kundali` | those, and muhurta, Tajika, prashna, KP, numerology, the SVG renderer | 81% |
| `panchanga` | those, and the rules, the interpretation, the dashas, the strengths, gochar and the chart's derived parts | 62% |

Only `panchanga` saves enough to be worth a second module: a third of
the code, and the 956 kB rule corpus with it. `kundali` and
`baseline-parity` would ship nearly the whole module under another name.
Each costs a build, a budget and an install check on every release,
so they wait until a measurement says otherwise.

The floor is the shared base: `teistro_core`, the façade, the FFI and
intl are a third of the module in every profile. Making that base
smaller is a separate lever from the profiles, and probably the larger
one.

## The design

1. **Features on the façade, forwarded.** A module family is a feature
   of `teistro` (`rules`, `muhurta`, `western`, …), each forwarded by
   `teistro-ffi` and the wasm crate. A profile is a feature that names
   families: `panchanga` names none of the optional ones and `full`
   names them all. The default is `full`, so every existing build stays
   as it is.
2. **The boundary keeps its shape.** Every entry point exists in every
   profile. A call into a family the build left out answers
   `TS_ERR_CAPABILITY`, naming the family and the profile ("this build
   is the `panchanga` profile, which leaves out `muhurta`"). A chart
   request asking for a left-out section is refused naming that field,
   never answered with the section missing. The generated glue is
   therefore one file for every profile, and only the `.wasm` differs.

   How the boundary stays whole without a `cfg` on every line:
   - The façade moves each family's methods into its own `impl` block
     in a file of its own, and gates the file. A Rust consumer who
     leaves a family out loses its methods and its re-exports, and the
     compiler names what is gone.
   - The FFI wraps each family's entry point or block in one macro,
     which compiles the body under the family's feature and a Capability
     refusal naming the family otherwise.
   - The chart blob keeps one schema. A left-out family writes its
     sections with no rows (`Writer::empty`), exactly what a request
     that did not ask for them writes, so a reader built for the full
     schema reads every profile's blob.
3. **One package, a subpath per profile.** `@teistro/sdk-wasm` stays the
   full module. `@teistro/sdk-wasm/panchanga` is the same layer with a
   loader that fetches `wasm/panchanga/teistro_wasm_bg.wasm`. A bundler
   that imports only the subpath ships only that module, which the
   bundler check in `check-wasm` already knows how to measure.
4. **The gates.**
   - fast-check builds each profile for `wasm32-unknown-unknown`,
     because a configuration no gate builds is already broken.
   - `size.json` gains a budget per profile, held both ways like the
     full one.
   - `check-wasm` runs a profile probe through the staged package. Each
     profile's calendar and almanac answers must equal the full
     module's to the bit, and one call into each left-out family must
     be refused as Capability.

## Order of work

1. `cargo xtask wasm-crates`, which reads a module's name section and
   prints the table above, so the numbers can be read again.
2. The `panchanga` family split in the façade and the FFI, behind
   features, with every refusal named. `full` stays the default.
   - **Done, first slice:** `kp`, `muhurta`, `numerology`, `prashna`,
     `remedies`, `svg`, `tajika` and `western`, each an optional crate
     of the façade and a forwarded feature of the boundary and the wasm
     crate. fast-check's `families` job builds the boundary and the wasm
     module with none of them, and `tests/families.rs` sends every
     record of a left-out family and reads its `CAPABILITY` back, naming
     the record. `families-are-forwarded` holds the three manifests to
     one set, read from the façade's optional crates.
   - **Second slice:** `rashifal`, whose entry point is its own. The
     ABI and key suites require `full`, and the `families` job lints
     every test target without it.
   - **Done: one `chart` family, not seven.** The crates the
     `panchanga` profile drops are not leaves: the chara kārakas, the
     Jaimini chart and the rashi dashas read the chart through
     `teistro_rules::RuleChart`, interpret depends on rules, and rules
     on the dashas and strengths. Gating each would make seven families
     that require each other, with refusals inside the chart path. So
     the chart area is one family, `chart`: the façade's `ChartArea`,
     the interpretation area and the crates only they reach (rules,
     interpret, dasha, strength, gochar, Hellenistic, matching). Every
     chart family above requires it (`kp = ["chart", "dep:teistro-kp"]`).
     At the boundary that is three entry points: `ts_chart_found`,
     `ts_chart_layout_row` and `ts_naam_milan`. The `panchanga` profile
     is then the calendars, the almanac, positions and intl, with or
     without `muhurta`.

     Building it moved what the almanac shares with the chart out of
     the chart module: the day's enums and values (`day.rs`), and the
     request struct (`chart_request.rs`), which the generated glue
     builds in every profile. The chart and naam blob schemas live in
     `schemas/charts.rs`. A context's own dasha systems are the chart
     area's, so `options.dashas_json` is refused by its field in a
     build without it. The façade's own tests and examples are written
     against the full surface and build only with `full`.
3. The `panchanga` module staged beside the full one, its subpath, its
   budget and its probe.
4. Measure the shared base by module, and decide whether to shrink it.

## Open questions

- Whether a native library should ship per profile too, as ADR-0005
  names them as native build targets. A native library is loaded, not
  downloaded, so its size matters far less; the wasm module comes first.
