# A graha's classical doshas and tattva

Status: `built`, 2026-11-01. Design for the consumer's SR-48
(`docs/consumers/addendum-3.md`), written before the code.

The application asks for each graha's tridosha and tattva weights from
BPHS ch. 3, marked as tradition and not as health. This page records
what the text gives and how it is kept from reading as health advice.

## What the text gives

The print in hand is the 1899 Gyan Press BPHS with the Subodhini
commentary (Nepal Sanskrit University scan, public domain). Its
Purvakhanda ch. 2, ग्रहप्रादुर्भाव, is the chapter the usual numbering
calls ch. 3. Verse numbers below are the print's own (C1108).

- **Tattva, v. 10.** Fire, earth, ether, water and air, in order, to
  Mars, Mercury, Jupiter, Venus and Saturn. The verse names none for
  the Sun, the Moon or the nodes (C1109).
- **Doshas, vv. 13–20**, one verse per graha:
  - the Sun is of pitta nature (v. 13);
  - the Moon is much vata and kapha (v. 14);
  - Mars is of pitta nature (v. 15);
  - Mercury has pitta, kapha and vata (v. 16);
  - Jupiter is of kapha nature (v. 17);
  - Venus has kapha in excess, with vata (v. 18);
  - Saturn is of vata (v. 19);
  - Rahu is of vata nature, and Ketu is like him (v. 20).

The OCR of vv. 14 and 15 is broken across a line. The readings above
are the verses' own words where the scan shows them (बहुवातकफः,
पित्तप्रकृतिकः), and the cruxes say so.

## No weights

The text names doshas, not weights. Venus is the only graha whose
verse ranks one dosha over another. A number such as 0.7 kapha and
0.3 vata would be invented, so none is shipped (C1108). What ships is:

- `classical_doshas`: the doshas the verse names, in its order;
- `classical_predominant_dosha`: the one the verse makes predominant.
  This is the only one named, or Venus's kapha. It is empty where the
  verse names several equally (the Moon, Mercury);
- `classical_tattva`: v. 10's element, empty where v. 10 is silent.

The catalogue already has `descriptors.element`, from the baseline
engine, for all nine grahas. It stays as it was. The two are told apart
by their names and their sources (C1109).

## Tradition, not health

The doshas are Ayurveda's words, and a field called `dosha` would
invite a reader to take it as a diagnosis. So both the names and the
docs say what the values are (C1110):

- the kind is `tridosha`, and its doc calls it a classical attribution
  of jyotisha texts, not health or medical advice;
- every attribute is prefixed `classical_`.

The values are catalogue attributes of `graha`. They go wherever the
catalogue goes: the Rust `Graha::attributes()`, `catalogue/catalogue.json`
and MCP's catalogue resource. The `tridosha` members are named in every
binding's catalogue and in `en-Latn` and `ne-Deva-NP`.
