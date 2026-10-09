# teistro-lalkitab

Lal Kitab's computable parts, read from the 1952 edition and cited by its
pages (`docs/03-design/lalkitab.md`):

- the **teva**, each planet's whole-sign house from the lagna, house `n`
  read as rashi `n`;
- each planet's **dignities** (pakka ghar, exalted, debilitated, own), its
  regard for its house's owners, whether it is **awake** and whether it is
  **kayam**;
- the **aspects**, forward only with their strengths, and the *yog
  drishti* relations of every house;
- the **masnui** planets two planets in one house make, the nine **rinas**
  and the ancestors' debt's first state, and the chart's own flags
  (ratandha, nabalig, the dharmi papis, sathi pairs);
- the **35-year cycle**, from the book's general start or the reader's;
- the **varshphal** annual chart, over a list the reader supplies and the
  crate checks.

The book is in copyright, so what ships is method in the SDK's own words.
The 120-year varshphal list is the author's table and is not shipped:
`VarshphalTable::from_rows` reads one and checks that every row is a
permutation and every twelve years a Latin square.
