# Extensibility

Status: `draft`, revised 2026-09-04 (rows aligned with the kernel designs
in `03-design/`). Every extension point is a registry on the context
populated at creation; nothing global, nothing discovered from the
filesystem.

## Extension points

| point | interface | what a consumer supplies | validation |
|---|---|---|---|
| ephemeris provider | vtable or host object implementing the port | positions, houses, ayanamsha, and optional features | capabilities checked; conformance kit available |
| calendar | `Calendar` trait or a data-driven month-length table | arithmetic or table calendar; lunisolar via the ephemeris | range and round-trip checks |
| timezone data | provider trait or a tzdb blob | newer tzdb | version stamp |
| geo data | provider trait | coordinates to zone, place catalogue | none |
| locale pack | `.tpack` blob | any language | `teistro-intl validate` at build; manifest and CRC at load |
| interpretation pack | `.tpack` blob | texts with citations | same plus citation presence |
| rule pack | `.rpack` blob compiled from YAML rules and tables (`03-design/rules-engine.md`) | yogas, doshas with cancellations and severity, classifying rules, cited lookup tables, muhurta rules, matching tables, remedy rules, Western configurations, custom yogas | schema, citations present, referenced keys and tables exist, acyclic references, classify arms exhaustive, positive and negative fixtures embedded |
| dasha system | a row over the udu or rashi kernel (`03-design/dasha-kernels.md`: seed and seed-to-lord map, lords and periods, sub-start, balance, year length, scale, applicability rule, citations, confidence mark) or, for the systems the schema cannot express, a trait implementation in Rust | new systems and school variants | the whole-table invariants (exact totals, every seed maps, orders visit every sign once, children sum to parents exactly) |
| house system | a trait implementation (cusps from ARMC, obliquity, latitude) or a provider capability | | degeneracy behaviour declared |
| varga scheme | a row over the varga kernel (`03-design/varga-kernel.md`: divisions, span rule, a linear map or an explicit table, citations) | custom D-N under a named convention and named school variants | spans sum to 30 degrees, entries in range, cyclic maps cover evenly, explicit and linear forms agree |
| bala scheme | a row over the strength kernel (`03-design/strength-schemes.md`: groups, component references with variants and weights, aggregation, required rupas) | school conventions | six groups exactly, no duplicate component, every variant implemented or refused at load |
| points | a declarative formula over positions and cusps (A+B−C style with day/night variants) | Arabic parts, sahamas, custom points | dependency check |
| composers | a narrative plan function (Rust) or a declarative plan (v1.x) | new prose shapes | keys exist |
| profiles | a settings record | school or product defaults | complete and valid |
| limits | per context | batch sizes, ranges, caches | bounds |

## The rule pack format

```yaml
pack: { id: "baseline-yogas", version: "1.0.0", sdk_catalogue: ">=1.0", licence: "..." }
rules:
  - key: GAJA_KESARI
    kind: yoga
    topic: [fortune, intelligence]
    sources: [{ text: "BPHS", ref: "36.14" }]
    when:
      all:
        - { in_kendra: { of: MOON, body: JUPITER } }
        - { not: { combust: JUPITER } }
    strength:
      base: 60
      add: [{ if: { exalted: JUPITER }, value: 20 }, { if: { dignity: { body: JUPITER, at_least: FRIEND } }, value: 10 }]
    cancel:
      - { key: GAJA_KESARI_BHANGA_DEBILITATED, when: { debilitated: JUPITER }, weight: 1 }
    tests:
      - { chart: "fixtures/chart-001", expect: { net: PRESENT } }
      - { chart: "fixtures/chart-007", expect: { net: ABSENT } }
```

Subjects may be references to derived points (`{ lord_of: 7 }`,
`{ arudha: 1 }`, `{ sphuta: BEEJA }`), predicates may look up cited tables
shipped in the pack, and a rule may classify (`outcome: classify`) or
grade instead of only firing (`03-design/rules-engine.md`). The predicate
vocabulary is closed and versioned; packs are compiled to a binary form
with resolved key ids; the engine evaluates compiled predicates over an
indexed chart state and can return a trace of every node.

## Versioning of extensions

Each pack and plug-in declares the SDK catalogue version range it targets.
A context refuses a pack outside the range with a structured error. Packs
carry content hashes that enter result provenance.

## Guidelines

Walkthroughs live in `09-guidelines/`: adding a language, a module, a
calendar, a dasha system, a rule pack, a provider.
