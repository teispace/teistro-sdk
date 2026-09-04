# Yogas, doshas and the rule engine

Status: `research`, 2026-09-04. Checked against the baseline engine's data-driven rule
corpus (562 yoga rules, 62 dosha rules, evaluated by one algebraic
evaluator) and the counts published by JHora (184 yoga types), PyJHora
(about 284) and Shri Jyoti Star (a custom yoga builder that searches
databases).

## The observation that shapes the module

Every yoga, dosha, muhurta rule, matching koota, horary consideration and
Western configuration is a **predicate over chart state**, optionally with a
**strength** and a **cancellation** clause. The baseline engine already treats yogas and
doshas this way: each rule is a record of an algebraic condition tree over
primitives such as "body in house from reference", "body in dignity",
"aspect between", "lord of house in house", "count of bodies in kendra",
with citations. The SDK generalises this into one rule engine with typed
primitives and several rule packs, so that a consumer can author rules
without touching the engine and Shri Jyoti Star's "custom yoga builder"
becomes a data format.

## Primitive vocabulary the engine needs (from reading the baseline engine's rules)

| primitive family | examples |
|---|---|
| placement | body in sign, body in house from lagna or Moon or Sun or any body, body in a varga sign or house, body in nakshatra or pada, body in degree band, in kendra, trikona, dusthana, upachaya, panapara, apoklima |
| lordship | lord of house N (with co-lords for Scorpio and Aquarius), lord in house M, lords exchanging, lord conjunct, lord aspected, lord's dignity, lord's varga placement |
| dignity and state | exalted, debilitated, own, mooltrikona, friend, enemy, combust, retrograde, in war, vargottama, gandanta, hemmed between benefics or malefics (kartari) |
| relationship | conjunction (same sign, optionally within orb), graha drishti (full or partial), mutual aspect, rashi drishti, argala, exchange (parivartana) |
| counting and comparison | number of bodies satisfying a predicate, benefic/malefic classification (natural and functional per lagna), strength comparisons (Shadbala, degree), ordering (who is stronger) |
| context | day or night birth, waxing or waning Moon, gender, lagna sign, tithi, nakshatra of Moon |
| cancellation | neecha bhanga rules, kemadruma cancellations, Kaal Sarp exceptions, Manglik cancellations (which are themselves rules) |
| strength | contribution weights and multipliers, minimum thresholds, "percentage of formation" |

## Yoga families to cover

| family | count in classical sources | baseline | notes | tier |
|---|---|---|---|---|
| Pancha Mahapurusha (Ruchaka, Bhadra, Hamsa, Malavya, Sasa) | 5 | yes | strength depends on kendra and dignity | P0 |
| Chandra yogas (Sunapha, Anapha, Durudhara, Kemadruma, Adhi, Gaja Kesari, Chandra Mangala, Amala) with Kemadruma cancellations | ~12 | yes | | P0 |
| Surya yogas (Vesi, Vosi, Ubhayachari, Budha Aditya, Nipuna) | ~6 | yes | | P0 |
| Nabhasa yogas (Ashraya 3, Dala 2, Akriti 20, Sankhya 7) | 32 | yes | | P0 |
| Raja yogas (kendra–trikona lord relationships, Dharma-Karmadhipati, Viparita Raja, Neecha Bhanga Raja, Parivartana Raja, and the specific named ones) | dozens | yes | strength and "which houses" reporting | P0 |
| Dhana yogas and Daridra yogas | dozens | yes | | P0 |
| Arishta and Balarishta yogas with their cancellations | dozens | yes | | P0 |
| Parivartana (Maha, Khala, Dainya) | 3 classes over 66 pairs | yes | | P0 |
| Sanyasa, Vahana, Vidya, Kalatra, Putra, Matru, Pitru, Bhratru yogas by topic | dozens | yes | topic classification is part of the rule record | P0 |
| Lunar and solar eclipse yogas (Grahan) | | yes (dosha) | | P0 |
| Kala Sarpa and its 12 named types, partial and complete, with exceptions | 12 | yes | ascending and descending; Kala Amrita variant | P0 |
| Manglik (Kuja dosha) with the house sets by lagna, Moon and Venus, cancellations, severity | 1 with many variants | yes (severity 0–100) | | P0 |
| Doshas: Pitru, Guru Chandala, Gandanta, Ganda Moola, Kalathra, Ghata, Shrapit, Angarak, Vish, Chandal, Sade Sati flag, Daridra, Kemadruma | ~20 | yes (62 rules) | | P0 |
| Tajika yogas (16: Ithasala kinds, Ishrafa, Nakta, Yamaya, Manau, Kamboola, Gairi Kamboola, Khallasara, Radda, Duphali Kutta, Dutthotha Davira, Tambira, Kuttha, Durapha) | 16 | partial | orb tables (deeptamsha) | P0 |
| Yogas in vargas (D9, D10 specific) | | partial | | P1 |
| BV Raman's 300 Important Combinations | 300 | overlap | PyJHora's 284 include these | P1 |
| Western configurations (grand trine, T-square, grand cross, yod, kite, mystic rectangle, stellium) | ~10 | no | orb-based rule pack | P1 |
| Horary considerations before judgement (Lilly) | 7 | no | | P1 |

## Yoga strength, timing and interpretation

- **Strength**: the baseline engine reports presence plus a strength derived from the
  participants' Shadbala and dignity. JHora reports presence per rule.
  Parashara's Light gives yoga pages with strength commentary. The SDK
  should compute a rule-declared strength formula and expose the
  contributing factors as data.
- **Timing**: when a yoga fructifies (dasha or antardasha of participants,
  transit triggers). The baseline engine's `YogaTimingComposer` produces this text from
  the active dasha chain; the SDK exposes the underlying relation (rule
  participants versus dasha lords) as data.
- **Search**: Shri Jyoti Star and Kala search a chart database for yogas.
  This is the batch form of rule evaluation and is a first-class API.
- **Custom rules**: a rule pack format (versioned, validated, cited) that
  consumers can author. See `02-architecture/08-extensibility.md`.

## Closing checklist

- Export the baseline engine's 624 rules to the SDK rule format mechanically and diff the
  evaluation results on 100 charts (this is the golden-vector set for the
  engine).
- Decide the strength model and document it with citations.
- Define benefic and malefic classification variants (natural, functional
  per lagna with the trikona override, Moon phase dependent for Moon and
  Mercury's association rule).
- Confirm Rahu and Ketu treatment in Nabhasa and Kala Sarpa rules.
