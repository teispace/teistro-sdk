# Adding a module

Status: `draft`, 2026-09-04. The checklist a pull request that adds a module
must satisfy.

1. **Research and design.** A research row in the feature universe (or a
   new page) and a design page in `03-design/` following the template.
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
   the baseline engine, PyJHora or hand-computed) with tolerances, property tests for
   invariants, snapshot tests for serialised output, cross-binding parity
   fixture.
7. **Localisation.** Every key the module emits has entries in every
   shipped locale, or the module documents that it emits none; composer
   plans have snapshots per language.
8. **Performance.** A benchmark in `tools/bench` with a budget; the size
   report shows the module's cost per profile.
9. **Docs.** Reference generated; a guide page with executed examples; the
   module catalogue row added.
10. **Extension.** If the module introduces a registry, the plug-in
    interface is documented with an example consumer registration.
