# Reception, measured

Status: `generated` by `cargo xtask reception` from Lilly's *Christian
Astrology* (1647) and the corpus's recorded births, 2026-10-02. Do not
edit: `check-reception` regenerates this page and fails on any
difference.

Crux C210 asks which receptions count and what they score. Lilly calls
it reception when two planets stand "in each others dignity", by any
essential dignity, and his table (p. 115) scores only mutual reception
by house and by exaltation (`essential-dignities.md` §Reception). The
SDK reports every pair whole and scores those two alone, apart from a
planet's own score. This page holds Lilly's worked examples to the
shipped tables and measures, at each of the corpus's 55 births under the
default request (Valens's horizon, Lilly's tables and scores), how often
each kind holds; the triplicity claim reads the same births under
Ptolemy's triplicities.

| proposed rule | verdict | measured |
|---|---|---|
| Lilly's three receptions (p. 112) hold as he prints them, under his own table (p. 104) | **holds** | 0 of 3 disagree; by house, by triplicity and by term |
| the triplicity example is a day chart's alone ("if the Question or Nativity be by day") | **holds** | by night it does not hold |
| how many pairs receive each other, and by what | **holds** | 260 pairs over 55 charts; mutual by house 12 (in 12 charts), exaltation 11 (in 10 charts), triplicity 24 (in 23 charts), term 23 (in 22 charts), face 28 (in 23 charts); 170 mixed (by no kind both ways) |
| every reported pair has each planet standing in at least one of the other's five dignities | **holds** | 0 of 260 disagree |
| under Lilly's triplicities a planet is received by any one kind by one partner at most, so a kind's score is earned once | **holds** | 0 of 1925 disagree |
| under Ptolemy's, a planet in water can be received by triplicity by two partners, Venus or the Moon and Mars, and both are reported | **holds** | 4 planets so received over the corpus |
| each planet's reception score is Lilly's house worth when received by house and his exaltation worth when by exaltation, and nothing else | **holds** | 0 of 385 disagree; 43 planets score by reception; 21 of them peregrine, which reception does not lift (C210) |

## What it means

Lilly scores two kinds and the SDK scores those two. Every other
reception, mixed or by a lesser dignity, is reported with both sides
whole, so a reader who scores more adds the kinds each pair carries
rather than the SDK giving a worth Lilly never printed. The last row
counts how many planets scored by reception are peregrine all the same:
that is how many totals the choice not to lift peregrine moves.
