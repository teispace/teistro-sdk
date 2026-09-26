# The arudha padas, measured

Status: `generated` by `cargo xtask arudhas` over the conformance
corpus's `houses.arudha_padas`, 2026-09-15. Do not edit: `check-arudhas`
regenerates this page and fails on any difference.

The corpus records the twelve padas of 71 charts, each with the sign it
first counts to and whether an exception moved it, beside the lagna and
every graha's sign they are computed from: 852 padas.

## What the corpus decides

| proposed rule | verdict | measured |
|---|---|---|
| a house's pada is the count from its sign to the sign's lord, counted again from the lord, with the catalogue's lords; one that falls in the house or the seventh from it moves to the tenth from itself | **holds** | 0 of 852 disagree; the exception moves 147 in the house and 153 in the seventh |
| the same with Ketu for Scorpio and Rahu for Aquarius | falsified | 123 of 852 disagree |
| the same with the stronger of the two lords by ch. 46's ladder, the nodes as co-lords (BPHS ch. 29 v. 7, `jaimini.node_co_lordship = BOTH`) | falsified | 46 of 852 disagree |
| the same with the exception moving a pada to the tenth from the **house** | falsified | 153 of 852 disagree |
| the same with no exception | falsified | 300 of 852 disagree |
| the arudha lagna is the first house's pada | **holds** | 0 of 71 disagree |

## What it means for the module

**The padas are a function of the lagna's sign and the grahas' signs**,
so they are points and not a school's module: `teistro-points` computes
them and the sign-based dashas read the first. The lords are the
catalogue's, which is the reading the recording engine takes for the
padas even though its Jaimini dashas count Scorpio and Aquarius to Ketu
and Rahu first; that difference is the engine's, measured here and in
`rashi-dashas-measured.md`.

**The exception counts from the pada**, so a pada in the seventh lands
in the fourth house; the tenth from the house is refused wherever the
exception applies to the seventh.

**v. 7 reaches the padas only through the co-lordship.** Its
द्विनाथ, a sign with two lords, counted up to the
stronger, is a question only where a node co-lords Scorpio or Aquarius.
Under `jaimini.node_co_lordship = NONE`, the default, each has one lord
and it is the catalogue's, so the shipped padas are the verse's as well
as the corpus's; a consumer who makes the nodes co-lords gets the
stronger lord, and the row above counts what that moves against a corpus
that never does (crux C135). The count itself is the shipped
`points::arudha::arudha_by`, so a change to it moves this page.
