# The shared validator corpus

Status: `implemented`, 2026-10-25. The quality bar's "type safety per
binding" row ([`05-testing/01-quality-bar.md`](../05-testing/01-quality-bar.md)),
its last part, and ADR-0023's sixth point: one shared corpus of valid and
invalid inputs runs through every binding's validators and the Rust
constructors, and they must agree.

## 1. What is validated twice

The description brands three quantities on `ObserverC`: `longitude`
[-180,180] deg, `latitude` [-90,90] deg and `altitude` [-500,12000] m.
Every binding generates its constructor from that range: Node's
`latitude()`, Python's `Latitude`, Dart's `Latitude` and Java's
`Latitude`. The Rust newtypes in `teistro_core::quantity` keep their
own constants. So the four bindings agree with the description by
construction, and nothing held the description to Rust. A range edited
on one side would let a binding accept a latitude the library then
refuses, or refuse one it accepts.

## 2. The corpus

`cargo xtask gen ffi` writes `idl/validators.json` from the description,
beside `api.json`, so `check-gens` holds it to the brands. For each brand
it lists:

- **valid**: both bounds, the double just inside each, and a value
  inside the range;
- **invalid**: the double just outside each bound, NaN, both
  infinities, and ±1e308.

Each value is written as a string. JSON cannot hold NaN or an infinity,
and every language's standard parser reads `"NaN"`, `"Infinity"`,
`"-Infinity"` and a shortest decimal to the same double: Rust's `parse`,
JavaScript's `Number`, Python's `float`, Dart's `double.parse` and Java's
`Double.parseDouble`. The neighbouring doubles are where an off-by-one
lives, such as `<` written for `<=` or a bound written as `90.0001`. A
round value would never find one.

## 3. Who reads it

| reader | test | gate |
|---|---|---|
| Rust constructors | `crates/ffi/tests/validators.rs` | fast check |
| Node | `bindings/node/test/validators.test.mjs` | `check-node`, verify |
| Python | `bindings/python/tests/test_validators.py` | `check-python`, verify |
| Dart | `bindings/dart/test/validators_test.dart` | `check-dart`, verify |
| Java | `bindings/java/test/com/teispace/teistro/ValidatorsTest.java` | `check-java`, verify |

Each test finds the corpus by walking up from its own file to the
directory holding `idl/validators.json`. It fails if it finds none, and
fails a brand whose constructor it does not know, so a fourth brand
cannot be skipped by every reader at once.
