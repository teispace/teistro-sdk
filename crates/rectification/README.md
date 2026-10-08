# teistro-rectification

The parts of a birth window that the verses testing a birth time leave
standing (`docs/03-design/rectification.md`).

A birth record gives a window, not an instant. `narrow` cuts the window
wherever a clause changes and judges each run once. It answers the runs
a bar left standing and the runs it removed, each with the clauses that
held and the verses they read. It never picks a single minute.

The first stage is **the purifier** of BPHS ch. 2 vv. 67–78. A human
birth's lagna must stand in the sign of the pranapada, of Gulika or of
the Moon, or in a trine of one. Each crux the verses leave open is a knob
on `Rules`:

- the pranapada's reckoning (`VERSE`, `PRINTED_EXAMPLE` or `SDK_POINT`);
- Gulika's end of Saturn's eighth (`END` or `START`);
- v. 76's extension;
- the species;
- bar or weight.

The kernel reads no ephemeris. A `Sky` supplies the lagna, the Sun, the
Moon and the day.
