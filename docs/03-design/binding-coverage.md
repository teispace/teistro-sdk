# Binding coverage: each binding's own layer, measured and floored

Status: `design`, 2026-10-28. Derives from the quality bar's coverage row
([`05-testing/01-quality-bar.md`](../05-testing/01-quality-bar.md),
"coverage floors"), whose **not yet enforced** half this closes: "80% on
binding ergonomic layers". The crates' half is built
([`coverage-measured.md`](../05-testing/coverage-measured.md),
`xtask/src/coverage.rs`); this page extends the same record to the
bindings rather than starting a second one.

## 1. What is measured

A binding's layer is the code in its language that a consumer imports:
the generated decoders, records and key sets, and the hand-written
ergonomic layer over them. Both are counted. A generated file is code
the binding ships and its tests must reach, and the crates' rule that
leaves generated *tables* out does not carry over: a binding's generated
files are readers, not data.

| binding | counted | tool | lines | branches |
|---|---|---|---|---|
| Node | `bindings/node/lib/**/*.js` | Node's own `--experimental-test-coverage`, LCOV reporter | yes | yes |
| Python | `bindings/python/teistro/**/*.py` | coverage.py, `--branch`, its LCOV report | yes | yes |
| Dart | `bindings/dart/lib/**/*.dart` | `dart test --coverage-path --branch-coverage`, LCOV | yes | yes |
| Java | `bindings/java/src` and `generated` | JaCoCo's agent, its XML report | yes | yes |

The C binding is the header and the library, which the crates' row
already measures; wasm's layer is Node's files run in a browser, so
Node's figure stands for it. Tests, examples, benches and the typecheck
consumers are not counted, as a crate's are not.

Each tool's own format is read, never re-implemented: three write LCOV
(`LF`/`LH` lines, `BRF`/`BRH` branches per file), JaCoCo writes XML with
its `LINE` and `BRANCH` counters. Node needs nothing installed; its
coverage is the runtime's own. coverage.py and the JaCoCo agent are
pinned (`xtask/coverage-requirements.txt`; a Maven coordinate with its
SHA-256), so a tool's upgrade, which moves its counts, is a commit. Node
and Dart count with the runtime itself, so their version is the
runtime's, which each `.tool` file names.

## 2. How a run is asked for

`check-node`, `check-python`, `check-dart` and `check-java` each read
`TEISTRO_BINDING_COVERAGE`. Unset, each runs as today. Set to a
directory, each runs its suite under its tool and writes its report
there (`node.lcov`, `python.lcov`, `dart.lcov`, `java.xml`). One
variable, not a flag per check, so the chain and verify ask for all four
the same way, and a check that ignores it is visible as a missing file.

## 3. The record and the gate

`docs/05-testing/coverage.json` gains a `bindings` map beside `crates`,
each entry a line floor and, where the tool counts them, a branch floor,
with the tool and its version that counted it (each tool counts its own
way, as each llvm does). The rules are the crates':

- `cargo xtask coverage --bindings DIR` raises each binding's floors to
  the run's figures, rounded down to a whole percent, and never lowers
  one; a record counted by another tool version is re-based;
- `cargo xtask check-coverage --bindings DIR` fails a binding under
  either floor, a floored binding with no report, and a report with no
  floor;
- `check-coverage` without a directory holds the floors to `main`'s, so
  a floor lowered by hand fails;
- the page `coverage-measured.md` gains a bindings table.

The 80% is the target, not the floor: a binding starts at its measured
figure and its floor only rises. The quality bar row then says which
bindings are at 80% and which are not, from the record.

## 4. Where it runs

Verify's `bindings` job on its `linux-x64` row sets the variable for the
four checks, uploads the directory, and runs `check-coverage --bindings`.
Other rows run the checks as today: the figure is a property of the
layer, not of the platform, and one row keeps the job's time. A local
run is the same command with the variable set.

## 5. Work items

1. LCOV and JaCoCo readers, the `bindings` section of the record, the
   page table, `coverage --bindings` and `check-coverage --bindings`,
   with unit tests on small fixtures of each format.
2. Node under its own coverage in `check-node`.
3. Python under coverage.py in `check-python`, the pinned requirement.
4. Dart's `--coverage-path --branch-coverage` in `check-dart`, on the
   suite and not the leak checks, which read the allocator.
5. Java under the pinned JaCoCo agent in `check-java`.
6. Verify's `linux-x64` row, the first record, the quality bar row.
