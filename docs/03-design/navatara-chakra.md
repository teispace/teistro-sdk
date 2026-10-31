# The navatara chakra

Status: `built`, 2026-10-31. Written before the code, for the
application's SR-44 (addendum 3): the navatara (tara) chakra of a birth
star.

## 1. What an almanac prints

The navatara chakra lays the 27 stars out from a native's birth star in
three rounds of nine. Each star is named by its tara, the count from the
birth star reduced by nine: Janma, Sampat, Vipat, Kshema, Pratyak,
Sadhana, Naidhana, Mitra and Parama Mitra. The three rounds are the
janma, anujanma and trijanma *paryāya*, which begin at the 1st, 10th and
19th star. Six stars carry a name of their own besides their tara.

The SDK already reads one cell of the chakra: `tara(birth, day)`
(`muhurta.md`, Raman, *Muhurtha*, ch. III) counts a day's star from the
birth star and answers its count, tara and round. The chakra is that
count over every star, so it is a generator, not a table: nothing in it
is transcribed but the names.

## 2. The special stars

*Muhurta Chintamani*, Nakshatra-prakarana v. 43 (1954, p. 82), names
stars at fixed counts from the birth star, the 10th, 16th, 18th, 23rd
and 25th, and the Piyushadhara's quotations of Narada and the Dipika name them:

| Count | Name | Round and tara |
|---|---|---|
| 1 | *janma* | janma, Janma |
| 10 | *karma* | anujanma, Janma |
| 16 | *sāṅghātika* | anujanma, Naidhana |
| 18 | *samudāya* | anujanma, Parama Mitra |
| 23 | *vaināśika* | trijanma, Pratyak |
| 25 | *mānasa* | trijanma, Naidhana |

The verse goes on to three stars of a king (*jāti*, *deśa*,
*abhiṣeka*), which are counted from a caste, a country and a
coronation the SDK is not given; they are not shipped and wait in crux
C1053 beside the verse.

The verse names the stars by their counts and gives no worked example.
The tara half of the chakra keeps Raman's (ch. III): born in Ashwini,
Shravana is the 22nd, Kshema.

## 3. The shape

`teistro_muhurta::tara::navatara(birth)` answers the 27 rows in the
order the count reaches them, from the birth star: each
`{nakshatra, count, tara, cycle, special}`, `special` one of the six
above or null. The rows are `tara(birth, star)` for every star, so the
chakra cannot disagree with tarabala.

At the surface it is the record `almanac.navatara`, asked as
`{"birth": "<nakshatra key>"}` and answered as the list, at the C
boundary `ts_navatara` (JSON in, JSON out, the `muhurta` family), and in
every binding the almanac area's `navatara(birth)`. The agent server
reaches it as the record's tool. It reads no sky, so it needs no place,
date or ephemeris.

## 4. Tests

- `tara` unit tests: every row is `tara(birth, row.nakshatra)`; each
  star appears once; the six special stars at their counts from every
  birth star; Raman's example inside the chakra; the round boundaries at
  the 10th and the 19th.
- The façade's request: the record read and refused (an unknown key, a
  star that is not a key), and the answer the kernel's.
- The C boundary: the JSON the kernel writes; a refusal named under
  `navatara`; the family refused as `CAPABILITY` in a build without
  `muhurta`; nothing held after a call.
- Every binding's method through each parity runner's `navatara` line.
