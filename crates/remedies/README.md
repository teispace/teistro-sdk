# `teistro-remedies`

Remedies as the Teistro SDK reads them, from the texts that prescribe
them. The first layer is **functional nature**: what a lagna's
lordships make of each of the seven grahas that own signs.

- ***Laghu Parashari*** (*Uḍudāya-pradīpa*, the 1894 Khemraj print)
  vv. 6 to 25: a trikona lord is good, the lords of 3, 6 and 11 are
  evil, the 8th lord is not good unless it is the lagna lord or a
  luminary, a kendra lord loses its natural nature, the 2nd and 12th
  act by association, one graha owning a kendra and a trikona is a
  yogakaraka, and the 2nd and 7th are the maraka houses.
- **BPHS** ch. 13 (the 1923 print) restates the rules and voids the
  lagna lord's own 6th or 8th (v. 12).

Every rule that holds for a graha is reported as a clause with its
verse, and the summary nature is derived by the text's precedence. The
baseline engine's reading (the lords of 6, 8 and 12 bad) is offered as
the `BASELINE` scheme of the same rules, so a migrating consumer gets
the same sets and can see where they part from the text.

The design is `docs/03-design/remedies.md`.
