# Extensibility

Status: `draft`, 2026-09-04. Every extension point is a registry on the
context populated at creation; nothing global, nothing discovered from
the filesystem.

## Extension points

| point | interface | what a consumer supplies | validation |
|---|---|---|---|
| ephemeris provider | vtable or host object implementing the port | positions, houses, ayanamsha, and optional features | capabilities checked; conformance kit available |
| calendar | `Calendar` trait or a data-driven month-length table | arithmetic or table calendar; lunisolar via the ephemeris | range and round-trip checks |
| timezone data | provider trait or a tzdb blob | newer tzdb | version stamp |
| geo data | provider trait | coordinates to zone, place catalogue | none |
| locale pack | `.tpack` blob | any language | `teistro-intl validate` at build; manifest and CRC at load |
| interpretation pack | `.tpack` blob | texts with citations | same plus citation presence |
| rule pack | `.rpack` blob compiled from YAML rules | yogas, doshas, muhurta rules, matching tables, remedy rules, Western configurations, custom yogas | schema, referenced keys exist, predicate depth limit, test cases embedded |
| dasha system | a declarative record (seed kind, sequence, lengths, ordering, applicability rule) or, for exotic systems, a trait implementation in Rust | new systems | tree invariants tested (children span parents) |
| house system | a trait implementation (cusps from ARMC, obliquity, latitude) or a provider capability | | degeneracy behaviour declared |
| varga scheme | a declarative mapping (division count, sign mapping rule) | custom D-N and named variants | bijection checks |
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
      any: [{ debilitated: JUPITER }]
    tests:
      - { chart: "fixtures/chart-001", expect: { present: true } }
```

The predicate vocabulary is closed and versioned (see the yogas research
page); packs are compiled to a binary form with resolved key ids; the
engine evaluates compiled predicates over an indexed chart state.

## Versioning of extensions

Each pack and plug-in declares the SDK catalogue version range it targets.
A context refuses a pack outside the range with a structured error. Packs
carry content hashes that enter result provenance.

## Guidelines

Walkthroughs live in `09-guidelines/`: adding a language, a module, a
calendar, a dasha system, a rule pack, a provider.
