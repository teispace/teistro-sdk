# The Node binding

Status: `generated`, 2026-09-06.

What is built: the TypeScript surface, the catalogue's tables and the
result-blob decoders, all rendered from `idl/api.json` by `cargo xtask gen
ffi` and held equal to the boundary crates by `cargo xtask check-ffi`. The
native addon (napi over the C ABI) and the ergonomic layer over it follow;
until they land this package has types and decoders and no native call.

| file | what it is |
|---|---|
| `lib/generated.d.ts` | every enum as a string union with a `const` table beside it, every boundary struct as an interface, every result blob's decoded shape, the error class; types only, nothing at run time |
| `lib/catalogue.js` | the tables the declarations promise, one frozen constant per enum so a bundler keeps only what an application uses |
| `lib/blob.js` | one decoder per result blob, reading the `TSRB` layout into typed-array views over the blob's own bytes |
| `test/blob.test.mjs` | the decoders against blobs the library really produced, with Node's own test runner and no install |
| `typecheck/` | a consumer that type-checks at maximum strictness, where every wrong usage is a compile error the file asserts |

A catalogue member is its full key everywhere a string names it
(`'graha.SUN'`), which is what packs, fixtures and serialised results
carry; any other enum member is its name in kebab case (`'invalid-arg'`).

## Running the tests

```sh
cargo xtask check-node
```

That writes blob fixtures through the C ABI, runs the decoder tests with
Node, and type-checks the consumer when a TypeScript compiler is on the
machine. It needs Node, so it runs by hand and in the nightly matrix; the
fast check needs the Rust toolchain and nothing else (ADR-0014).

Never edit `lib/`: change the Rust source and regenerate.
