# Adding a module

Status: `draft`, revised 2026-09-04 (kernel-and-table steps added,
ADR-0017 and ADR-0018). The checklist a pull request that adds a module
must satisfy.

1. **Research and design.** A research row in the feature universe (or a
   new page) and a design page in `03-design/` following the template.
   For a family of variants, the design page is a falsification pass:
   every variant the catalogue names written as a row over the proposed
   kernel, the schema corrected wherever a variant refuses to fit, and a
   section on what resisted. A variant that needs code keyed to its own
   identifier is a kernel defect, not a special case.
1a. **Confidence marks.** Every row carries V, T or S with its citation
   (file and line for the baseline engine, chapter and verse for a text).
   Only V rows are implemented; T and S rows are registered identifiers
   that return `UNSUPPORTED (unsourced)` until sourced. A disagreement
   with a third-party implementation goes to the cruxes page, never into
   the row.
1b. **Whole-table invariants.** One test runs the kernel's invariant list
   over every row (exact totals, complete maps, orders that visit every
   sign, spans that sum, children that sum to parents).
2. **Catalogue.** New keys, ids and attributes added to `catalogue/` with
   citations; the key gate passes.
3. **Crate.** `crates/<module>/` with `#![forbid(unsafe_code)]`, an explicit
   dependency list that keeps the DAG acyclic, features for optional
   pieces, and documentation on every public item.
4. **Settings.** Any new knob added to the settings model with a default in
   every shipped profile and a note in the changelog.
5. **API.** C ABI entry points in `ffi` following the conventions; the IDL
   re-extracted; bindings regenerated; ergonomic wrappers added in every
   binding with the same names; the parity gate passes.
6. **Tests.** Unit tests with classical examples, golden vectors (from
   the baseline engine, PyJHora or hand-computed) with tolerances and the
   settings hash asserted, property tests for invariants with boundary
   generators, snapshot tests for serialised output, cross-binding parity
   fixture, compile-fail tests for any new newtype, and the module's rows
   in the cross-architecture determinism matrix.
7. **Localisation.** Every key the module emits has entries in every
   shipped locale, or the module documents that it emits none; composer
   plans have snapshots per language.
8. **Performance.** A benchmark in `tools/bench` with a budget, an
   instruction-count benchmark for the pull-request gate, an allocation
   count for hot paths; the size report shows the module's cost per
   profile.
8a. **Calculation version.** The pull request states whether any numeric
   output moved and bumps the calculation version with a Numbers entry
   if so (ADR-0020).
9. **Docs.** Reference generated; a guide page with executed examples; the
   module catalogue row added.
10. **Extension.** If the module introduces a registry, the plug-in
    interface is documented with an example consumer registration.
