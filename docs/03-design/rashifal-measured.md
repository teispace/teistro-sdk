# Rashifal, measured

Status: `generated` by `cargo xtask rashifal`. Do not edit:
`check-rashifal` regenerates this page and fails on any difference.

The design this measures is `rashifal.md`. **No text scores a rashifal
and nothing records one**, so the kernel is held to the gochar it is
built from by the crate's tests, and this page counts what each of the
baseline engine's choices moves (C361), one at a time over the same sky:
every day from July 2026 to June 2027 at Kathmandu, a year that holds
the nodes' ingress and Saturn's, read for all twelve signs under the
default profile.

## 1. The baseline's tables (D1 to D3)

Each day's reading at sunrise judged again by the baseline engine's
vedha tables, `baseline_gochar`: Venus's 11th and 12th exchanged, the
nodes' hybrid vedha, and the Sun and the Moon its only exemption, with
every node obstructing. 2387 of 39 420 verdicts move, by graha:

| graha | verdicts moved, of 4380 |
|---|---:|
| Sun | 188 |
| Moon | 94 |
| Mars | 0 |
| Mercury | 110 |
| Jupiter | 0 |
| Venus | 451 |
| Saturn | 36 |
| Rahu | 626 |
| Ketu | 882 |

The daily score moves on 1873 of 4380 sign-days, by 16 at most.

## 2. 06:00 against sunrise (C358, B5)

The same days read at 06:00 at +05:45, the baseline's clock, under
the text's tables. The Moon is in another sign on 3 of 365 days,
37 of 39 420 verdicts move, and the daily score moves on 48 of 4380
sign-days, by 9 at most.

## 3. The events (C360, B1 to B4)

Every Sunday-to-Saturday week wholly inside the year (51) and every
month (twelve), each with its ingresses and stations of every graha but
the Moon. The SDK finds 128 events, each counted once in every period
that holds it, and the baseline's scan cannot report 64 of them, each
counted under the first reason that holds:

| why the baseline cannot report it | events |
|---|---:|
| a node's ingress (B1) | 4 |
| in a week, a graha its weekly scan leaves out, or a station | 57 |
| in a week, Saturn's ingress (B2) | 1 |
| after 06:00 on the period's last day (B3) | 2 |

Of the events found, 26 fall on another UTC date than their Kathmandu
one, which is the date the baseline prints (B4).

## 4. The panchanga's day (B6)

Each month scored with its reference day's panchanga, the SDK's,
and with its first day's, one day the baseline can take it from:
120 of 144 sign-months score otherwise.

## 5. What this pass decides

| proposed rule | verdict | measured |
|---|---|---|
| every event a period reports falls between local midnight of its first day and local midnight after its last | **holds** | 0 of 128 disagree |
| the baseline engine's vedha tables give the text's verdicts | falsified | 2387 of 39420 disagree |
| reading at 06:00 rather than at sunrise moves no verdict | falsified | 37 of 39420 disagree |
| the baseline engine's scan reports every event a period holds | falsified | 64 of 128 disagree |
| a month scores alike whichever of its days the panchanga is taken from | falsified | 120 of 144 disagree |

Four of the five claims are falsified. The first holds the façade's own
window; each of the others is one of the baseline engine's choices, held
against the SDK's default over the same sky.
