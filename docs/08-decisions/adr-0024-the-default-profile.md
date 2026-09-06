# ADR-0024: The default profile is the texts as read, and inherits nothing else

Status: accepted (maintainer, 2026-09-07)
Date: 2026-09-07
Question: Q34

## Context

A context whose options name no profile gets one anyway. Whatever it is
becomes the settings behind every chart nobody thought about: the
quick-start example, the first call in a notebook, the test somebody
wrote before reading the settings page. It is the one profile that is
chosen by not choosing, so it has to be the one that needs the least
defending.

Two candidates were on the table. `nepali-default` is the product's
charts: what the baseline engine persisted, with Nepal's civil calendar
and eras. `parashari-classical` is described as "the texts as read, with
nothing of one country's practice in it".

The choice is not cosmetic. `frame.centre` alone decides whether the
Moon is seen from the Earth's centre or from the birthplace, and the
corpus records both readings of the same six charts
(`fixtures/baseline/variants/*--geocentric.json`):

| what moves | how far |
|---|---|
| the Moon's sidereal longitude | up to **39.1′** (0.65°) across the six pairs |
| the Sun | 0.135′ |
| Venus, Mars | 0.246′, 0.060′ |

A pada is 200′. Two of the six charts change a classification: **c049**'s
Moon moves from nakshatra 20 to 21 — which changes the **Vimshottari
mahadasha lord**, and with it the whole dasha tree — and **c050**'s Moon
moves from pada 2 to pada 3. These are recorded facts about recorded
charts, not estimates.

The SDK's own frame completion does not apply the topocentric step yet
(it is deferred to Phase 3), but a provider that can, does, under the
`prefer-native` override policy the profiles set. So the setting already
decides what a consumer of a capable provider gets, and it will decide
what everyone gets once Phase 3 lands. A chart stored today under a
default that later starts applying parallax is a chart that changed
without anyone touching it.

**The defect this question exposed.** `parashari-classical` was declared
with `base: Some("nepali-default")`, so it resolved to Nepal's civil
calendar, the baseline engine's topocentric centre and its synthesised
polar days — none of which is in any text. Its own documentation said
the opposite. A profile whose name is an argument for choosing it must
be what the name says.

## Decision

**`parashari-classical` is the default**, and it is rebased onto the root
so that it is what it claims. Its base becomes `None`; it patches the
root with the four knobs that define it, each cited:

| knob | value | source |
|---|---|---|
| `houses.chalit_system` | `SRIPATI` | BPHS, the Sripati bhava |
| `day.ghati_reckoning` | `PROPORTIONAL` | the day divided by its own arcs, as the ishta-kaal is reckoned |
| `jaimini.chara_karakas` | `EIGHT` | Jaimini Sutras 1.1.10–18 |
| `state.combustion_orbs` | `SURYA_SIDDHANTA` | the text's own orbs where it gives them |

Everything else comes from the root, which is itself fully cited: sidereal,
Lahiri, mean node, **geocentric**, whole-sign placements, Amanta months,
the Gregorian civil calendar, and `UNDEFINED` for a polar day.

Four values therefore change, all of them by ceasing to be inherited:

- **geocentric** rather than topocentric. The citation for topocentric was
  "the baseline engine's persisted chart settings" — one implementation's
  choice, not a text. The classical graha sphuta is computed from the
  Earth's centre; the Surya Siddhanta's lambana is an eclipse correction,
  not a natal one. A default that differs from every published table by
  two thirds of a degree of Moon is a default nobody can check the SDK
  against.
- **the Gregorian civil calendar** rather than Bikram Sambat, and the eras
  Vikrama, Shaka and Kali rather than those plus Nepal Sambat. Both are
  one country's practice. The *lunar* month stays Amanta, which is the
  astrological one.
- **`UNDEFINED` polar days** rather than `NEAREST_EVENT`. The texts define
  no sunrise where the Sun does not rise. Reporting that a day has no
  boundary is the honest answer; synthesising one is a claim the tradition
  does not make.

`nepali-default` is unchanged and keeps all four: it is the profile that
reproduces the product's charts, and every one of its values is cited to
the engine whose charts it reproduces. A consumer who wants them names
it, which is one argument.

## Consequences

- The settings hash of `parashari-classical` changes, which is how a
  change of defaults is meant to be visible (`03-design/settings-and-profiles.md`).
  No number the SDK computes today moves, because the topocentric step is
  not built and no other changed knob feeds the astronomy; what moves is
  what a capable provider is asked for, and what everything computes once
  Phase 3 lands.
- A caller at a polar latitude who named no profile now gets a reported
  absence rather than a synthesised day. One knob, or `nepali-default`,
  restores the old behaviour.
- The profile's `version` is bumped to 2. A profile's version bumps when a
  default changes, and this is that.

## Alternatives considered

**`nepali-default` as the default.** It is a good profile and a bad
default: it is defined by an implementation's persisted settings and one
country's calendar, so a consumer who never chose would be silently
placed inside both. A default should be the least-committed answer
available, not the most useful one for the first market.

**Leaving `parashari-classical` on its `nepali-default` base.** That
keeps the inheritance convenient and the documentation false. The four
inherited values are precisely the ones a reader would assume a profile
called "the texts as read" does not have.

## Evidence

`fixtures/baseline/charts/` against `fixtures/baseline/variants/*--geocentric.json`
(the six pairs, measured); `03-design/settings-and-profiles.md` §3;
`03-design/astro-timescales-and-frames.md` §4 for the completion step that
is not built; `crates/core/src/settings/profiles.rs` for every citation.
