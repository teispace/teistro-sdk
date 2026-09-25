# teistro-ephemeris-kit

The provider conformance kit of the Teistro SDK
(`docs/03-design/ephemeris-port-and-adapters.md`, §9): the checks every
adapter and every built-in tier must pass, under one published set of
bounds and never per provider, with a machine-readable report; the timing
rows every provider is measured on; and the runner the kit binaries
share. It runs against anything that implements the port, native or
through the C vtable.

```sh
cargo run --release -p teistro-ephemeris-kit            # the test provider, in CI
cargo run --release -p teistro-ephemeris-kit -- --out target/kit
```

An adapter's binary opens its provider, adds its direct-binding row and
calls `runner::run` (`adapters/ephemeris-teimeris/rust`), then
`runner::charts` for the two checks made through the façade:

- `sdk_only`: under the `sdk-only` policy a chart is the provider's
  native positions and nothing else it offers, byte for byte;
- `corpus`: the conformance corpus's recorded charts founded over the
  provider, under the band its class is given, in the corpus's own report
  format, every miss one of `corpus::KNOWN` and every entry of it used.

```sh
cargo run --release --manifest-path adapters/ephemeris-teimeris/rust/Cargo.toml \
  --bin teistro-ephemeris-teimeris-kit -- \
  --corpus fixtures --class same-ephemeris --out target/kit
```

`--native-frame-only` founds the charts over the provider reduced to its
native frame, which measures the SDK's completion from its positions
rather than the engine's own frames.
