# `teistro-muhurta`

Muhurta: an elected time judged clause by clause, as the texts judge it
(`03-design/muhurta.md`).

Raman's *Muhurtha* holds that no moment is free of every defect, so the
crate reports **clauses** — one named condition from a source and the
interval it held over — rather than a score. What is built is a day's
clauses: the limbs its rules reject (tithi, nakshatra, yoga, karana,
vara), the special yogas the almanac found, and, against a native,
Tarabala and Chandrabala. The rules are data (`DayRules`), Raman's one
such value, and the Chandrabala table is the caller's to name because
the sources disagree on it (crux C160). An instant's clauses read the
lagna and the grahas' houses from it: the Mahadoshas that turn on them
and the neutralisations. The windows cut a day wherever any clause can
change — the lagna's navamsa and sign, found by search, the limbs, the
kaalas, the choghadiya, the horas and the muhurtas — and the lagna
tyajya is judged on the signs that rise whole. The season's blackouts
and the search follow (`muhurta.md` §6).
