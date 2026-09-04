# Adding a dasha system

Status: `draft`, 2026-09-04; worked example after Phase 5. A dasha system
is a row over a kernel (`03-design/dasha-kernels.md`), so adding one is
adding cited data, not code.

1. **Identify the kernel.** Seeded by a nakshatra, tithi, yoga or karana
   with proportional cycles: the udu kernel. A sign progression: the rashi
   kernel. A compression of another system into a shorter cycle (annual
   systems, Tribhagi): the scale decorator over that system's row. Several
   progressions at once: a composition. If none fits, stop and open a
   design question; a trait implementation is the last resort and the
   kernel's kill criterion applies (ADR-0017).
2. **Write the row** in the pack (`packs/dasha/<pack>.yaml`): id, seed,
   seed-to-lord map (reference, direction, span, offset, overflow), lords
   and periods (or the chart query), sub-start rule, balance method, year
   length, scale, applicability rule (a rules-engine rule key), sources
   and the confidence mark. For a rashi system: start rule, order rule,
   sequence direction, period-length rule, sub-progression rule,
   exceptions.
3. **Cite.** A primary text with chapter and verse, or the baseline
   engine's constants file and line. A row without a citation is marked
   S and registered only; it ships as `UNSUPPORTED (unsourced)`. A value
   read from a third-party implementation is a cross-check, never the
   source (ADR-0018, `CLEAN_ROOM.md`).
4. **Run the table invariants** (`cargo test -p dasha`): they refuse a
   row whose periods do not sum to its total, whose map leaves a seed
   unmapped without an explicit overflow, or whose order does not visit
   every sign once.
5. **Fixtures.** At least one golden vector per system, cited (a text's
   worked example, reference software with its version, or the baseline
   engine), with the balance method and year length recorded; a
   boundary fixture at a period boundary.
6. **Localisation.** The system's name under `sdk.dasha.<id>` in every
   shipped locale; level names if the system's differ from the profile's.
7. **Docs.** The generated reference renders the row; add the system to
   the feature page's table with its tier.

A consumer follows the same steps against their own pack and registers it
at context creation; the same invariants run at load.
