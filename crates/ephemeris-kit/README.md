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
calls `runner::run` (`adapters/ephemeris-teimeris/rust`).
