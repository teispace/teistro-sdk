# Verification cruxes and open items

Status: `research`, living register started 2026-09-04. Every entry is a
question the evidence has not yet settled at rank 1 (ADR-0018): a text
that does not agree with itself, a third-party implementation that
disagrees with the baseline engine, a parameter no reference read so far
supplies, or a convention that has to be chosen for the arithmetic to
terminate. An entry is closed by a citation, never by a vote; until then
the affected row is marked T or S and ships as `UNSUPPORTED (unsourced)`
or with the baseline engine's value as the documented default.

| # | item | what is known | what is missing | rank of the evidence | action |
|---|---|---|---|---|---|
| C1 | Shashtihayani dasha years | the received text (BPHS ch. 46, verses to confirm) gives Jupiter, Sun and Mars 13 years and the other five lords 6, which sums to 69; the name means sixty | an edition or the Sanskrit that reaches 60 | rank 1 text disagreeing with itself | stays S; do not guess an allocation |
| C2 | Narayana antardasha order | the baseline engine uses twelve equal sub-periods in the sign's parity direction and is corpus-tested; a third-party implementation carries three 12×12 permutation tables (normal, and exceptions selected by Saturn and by Ketu) that no parity rule generates | attribution of the tables to Sanjay Rath, Narasimha Rao or a primary text | rank 2 versus rank 3 | baseline stays the default; `narayana-table` registered S; the kernel gains `Table` and `exceptions` regardless |
| C3 | Ashtottari applicability | a translation of BPHS reads "Rahu not in the lagna, in another kendra or trikona from the lagna lord"; the baseline engine's condition may not exclude Rahu in the lagna | a check of the baseline engine's applicability rule against the verse | rank 1 versus rank 2 | verify before the rule pack exports |
| C4 | Shatabdika Mars and Saturn years | the baseline engine follows the Chaukhamba reading (Mars 20, Saturn 30) and notes editions that swap them; totals are 100 either way so a total check cannot see it | which editions swap | rank 1 editions | two rows: default and `shatabdika-alt` |
| C5 | Ashtottari seeds outside the cycle | eight lords times three nakshatras cover 24 of 27; the baseline engine wraps Krittika, Rohini and Mrigashira to the start | a text that addresses it (conditional dashas may be expected to fail applicability for these births) | rank 2 choice | `overflow: wrap-to-start`, flagged in provenance |
| C6 | Year length per dasha system | the baseline engine runs the udu family at 365.25; six year-length variants exist (tropical 365.2422, sidereal 365.2564, savana 360, lunar 354.367, Gregorian 365.2425, nakshatra 324, the last from a third-party constant, verify); savana against 365.25 compounds to about 21 months over 120 years | which system classically takes which | rank 3 and general knowledge | a per-system column in the year-length table, resolved before any dasha conformance run |
| C7 | Seed references for tithi, yoga and karana seeded systems | the lord tables are the base system's; the cycles differ (30, 27, 60), so the reference index does not carry over | the reference per system | none read | rows stay T |
| C8 | Required rupas for the Sun | the baseline engine ships 5.0 attributed to Raman; the strengths research page lists 390 shashtiamsas (6.5 rupas) from BPHS | the verse, and whether Raman differs | rank 2 versus a rank 1 figure from memory | `parashari-baseline` ships the baseline engine's values; verify |
| C9 | Ayana and Yuddha bala group membership | inside Kaala bala in both references; some authorities place them outside the six | the authorities and verses | general knowledge | default inside Kaala; the other grouping is a scheme row |
| C10 | Saptavargaja bala variants | a third-party implementation has four; unattributed to schools | whether they are one algorithm with parameters or four | rank 3 | variant 1 ships; the rest registered S |
| C11 | Uchcha bala | a Saravali formula exists beside the BPHS one, hidden behind a flag in a third-party implementation | the Saravali verse | rank 3 pointer to a rank 1 text | second variant registered T |
| C12 | Hidden variants generally | variant counts taken from function names are lower bounds; flags hide variants | a read of each reference's constant tables, not only its functions | method | audits read data files as well as code |
| C13 | Arbitrary D-N vargas | there is no classical rule for an unattested division count | nothing to find; a convention must be chosen | none | cyclic parivritti as the recorded default; the result carries the convention |
| C14 | Counting from the end of an even sign in vargas | a third-party parameter its own author marks as not matching reference software | a text | rank 3, self-flagged | registered S, unimplemented |
| C15 | D30 spans | 5, 5, 8, 7, 5 degrees mirrored for even signs is the standard BPHS reading | variant editions were not checked | rank 2 | verify before the golden vectors |
| C16 | Sign taxonomy as longitude ranges | the human, watery, quadruped and insect classifications are longitude ranges within signs, not whole signs, in a third-party constant table; also tridosha nature, dry and watery signs | primary citations | rank 3 | catalogue attributes typed as ranges; cited before use in prashna and muhurta rules |
| C17 | Retrograde-specific combustion orbs | a separate orb table for retrograde planets appears in a third-party constant table | citation | rank 3 | a second orb table row, T |
| C18 | Deep exaltation tolerance | exaltation as a band (about one degree), not a point, in a third-party constant | citation | rank 3 | a settings knob with the baseline engine's behaviour as default |
| C19 | Apparent disc diameters | needed for graha yuddha winner rules and visibility-based combustion; the fixed classical orbs remain the default convention | the table and its source | rank 3 | planetary phenomena (elongation, magnitude, diameter) in `astro`; both conventions selectable |
| C20 | Tri-rashi day and night lords | a lordship scheme differing by day and night birth in a third-party table | citation | rank 3 | catalogue rows T |
| C21 | Surya Siddhanta modern peripheries | a modernised parameter set for the classical model exists as a second variant | citation and the parameter set's origin | rank 3 | a `siddhanta` variant row, S |
| C22 | Varnada lagna | five schools (Sanjay Rath, Jha-Pandey, Raman, Santhanam, Sharma) disagree and a third-party implementation carries all five | each school's text | rank 3 | five rows; the profile chooses; none is "the" Varnada |
| C23 | Tajika ladder | the technique runs year, month and sixty-hour chart; the baseline engine stops at the year | nothing to verify; a scope note | catalogue | month and sixty-hour charts in `tajika` |
| C24 | Sphutas | fourteen sphutas are absent from the baseline engine | their formulas with citations | rank 3 lists them | `points` rows T until cited |
| C25 | Karaka dasha lord order | ordered by chara-karaka strength: a chart query the udu kernel does not have | the text and the interaction with Ashtakavarga and Tara dashas | rank 3 | decided when the three are built together (ADR-0017) |
| C26 | Panchaswara dasha | no attested shape read here | a text | none | S |

## How to use this page

An implementer picks an item, reads the text, and closes it with a pull
request that adds the citation to the row and the fixture that proves it.
A reviewer for a tradition (GOVERNANCE.md) signs off on the closure. New
discrepancies found while reading a reference are added here first and
resolved second; the rule is that a question is recorded, never absorbed
into a table as if it were an answer.
