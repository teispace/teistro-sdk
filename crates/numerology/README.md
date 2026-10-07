# `teistro-numerology`

Numerology as the Teistro SDK reads it: a name and a civil date, and
no sky. Two systems are offered, each from the text that defines it.

- **Pythagorean**: L. Dow Balliett's letter cycle, *The Philosophy of
  Numbers* (1908), p. 17. Each word is reduced and the words added
  (pp. 18–19), 11 and 22 stop a reduction and 33 does not (pp. 30, 90),
  and the birth number reduces the month, the day and the year apart
  before adding them (p. 19).
- **Chaldean**: Cheiro's table, *Cheiro's Book of Numbers*, p. 70. A
  name's compound is the sum of its words' single numbers (pp. 71–72),
  no number is a master (p. 35), and the birth number is the day of the
  month alone (p. 93).

The baseline engine's readings that no public-domain text prints (a
33, the letter total as a compound, the digit sum of the whole date,
the vowel and consonant numbers) are offered as `BASELINE` values of
the same rules, so a migrating consumer gets the same numbers and can
see that they are not the texts'.

The design is `docs/03-design/numerology.md`.
