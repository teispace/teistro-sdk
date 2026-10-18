# The aspect configurations (the `western` module)

Status: `built`, 2026-10-18, in Rust (`teistro_western::configurations`,
`sdk.chart().western_configurations`) and across the boundary as the
chart-request record `westernConfigurations`, read back as each chart's
`westernConfigurations` in every binding.

A Western reader looks at a chart's aspects one pair at a time
([`western-aspects.md`](western-aspects.md)) and then at the shapes
several pairs make together: a **grand trine**, a **T-square**, a **yod**.
This page builds those shapes.

## What the sources decide, and what they do not

**No text before the twentieth century names a configuration.** Leo, the
source this module's aspects and orbs are taken from, reads aspects pair
by pair. The names grand trine, T-square, grand cross, yod ("finger of
God"), kite, mystic rectangle and stellium belong to twentieth-century
astrology. No rank 1 or rank 2 source in reach states them, so none is
cited for them. What the names mean is not in dispute, though: each is a
shape of aspects, and its geometry is the definition. So the geometry is
what ships, one statement per shape, and nothing interpretive comes with
it.

| shape | the aspects it is made of |
|---|---|
| grand trine | three bodies, each trine the other two |
| T-square | two bodies in opposition, a third (the **apex**) square to both |
| grand cross | four bodies in two oppositions, each square to both ends of the other |
| yod | two bodies in sextile, both quincunx a third (the **apex**) |
| kite | a grand trine, and a fourth body (the **apex**) opposite one of its three and sextile the other two |
| mystic rectangle | four bodies in two oppositions, their ends joined alternately by trine and sextile |
| stellium | at least `stellium_least` bodies in one sign |

## The rules

- **The orbs are the aspects' own.** A shape is read over the rows
  `western_aspects` answers under the caller's `AspectRequest`. It holds
  exactly when every aspect it is made of holds, under that request's
  orbs. Nothing here widens a trine because it is part of a grand trine.
  So Leo's orbs by default, Lilly's moieties with `AspectRequest::lilly`,
  and a caller's own `OrbModel` are all read the same way. The answer
  carries `widest_from_exact_deg`, its loosest aspect's gap, so a reader
  who wants tighter shapes filters on it.
- **The yod needs the quincunx**, which is among Leo's nine (p. 47, with
  a narrow orb) and not among Lilly's Ptolemaic five. Under Lilly's
  request no yod can hold, and the request's aspect list is where that is
  decided.
- **A stellium is read by sign**, the tropical sign whatever zodiac the
  chart was founded in, since this is Western doctrine. Its count is the
  knob `ConfigurationRules::stellium_least`, three by default, the
  commonest modern count; some authors ask four. Under three is refused,
  naming `stelliumLeast`, because two bodies in a sign are a
  conjunction's business. A stellium by house, which some authors also
  use, is not built.
- **A shape inside a larger one is reported, and names it.** A grand cross
  holds four T-squares, and a kite holds a grand trine. Each smaller shape
  lists the larger shapes holding all its bodies in `inside`. A reader
  who shows only the largest filters on an empty `inside`. Dropping them
  silently would decide that for every reader.
- **The bodies are the planets the aspects are read among**: the seven,
  and the outer three where the chart placed them. The nodes and the
  angles are not read, as `western_aspects` does not read them.

## Tests

- **The geometry, one test a shape** (`crates/western/src/configuration.rs`):
  a grand cross and its four T-squares, each marked inside it; a kite and
  its grand trine; a yod's apex; a mystic rectangle; a trine 9° wide
  making no grand trine under Leo's 8°; a stellium of three by sign that
  four refuses; `stelliumLeast` of two refused by name.
- **A real chart** (`crates/sdk/tests/western_aspects.rs`): Edward VII's
  nativity, Leo's own fixture. It holds two T-squares, Moon–Uranus with
  Jupiter at the apex and Venus–Pluto with Mars at the apex, and each is
  read back from the aspect table itself.

## What is decided and what is not

| | |
|---|---|
| **decided** | each shape's geometry, as above; the request's own orbs; a stellium by tropical sign; containment reported, not resolved |
| **a setting** | `ConfigurationRules::stellium_least`; the `AspectRequest` the aspects are read under |
| **across the boundary** | the record `westernConfigurations`, `{aspects, orbs, stelliumLeast}`, each optional and laid flat (`ConfigurationRequest`), so `{}` is Leo's nine under his orbs with a stellium of three; a refusal names its field under the root (`westernConfigurations.stelliumLeast`). The shapes come back in the `western_configurations` section as canonical JSON, bare catalogue keys, which every binding reads into its own members (`graha.SUN` in Node, `Graha.SUN` elsewhere). Proved by a test in each binding on Edward VII's two T-squares and by every parity runner |
| **not decided** | the stellium's count, which the modern sources differ on, and whether by sign or house. No text in reach settles either, so the count is a knob |
| **not built** | a stellium by house; the shapes Marc Edmund Jones named for a whole chart's distribution (bowl, bucket, locomotive and the rest), which are a different question from a shape of aspects |
