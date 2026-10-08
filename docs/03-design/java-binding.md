# The Java binding

Status: `draft`, written 2026-10-08, before any code, as every binding
page is. It derives from
[`ffi-abi-and-api-description.md`](ffi-abi-and-api-description.md) (the
ABI, the description, the result blob, the error record),
[`python-binding.md`](python-binding.md) (the binding nearest in shape:
a foreign-function layer over the same shared library, with no compiled
glue), [`surface-areas.md`](surface-areas.md) (the areas every binding
mirrors) and [`binding-surface-measured.md`](binding-surface-measured.md)
(what a binding must marshal, spell and lay out). Decisions it rests on:
ADR-0004 (one description, generated bindings), ADR-0007 (the C header is
the contract for Java through FFM), ADR-0014 (everything we author is
Rust except a binding's own layer), ADR-0023 (type safety in every
binding), ADR-0029 and ADR-0030 (plugins and the engine façade). The
roadmap item is `07-roadmap/00-roadmap.md`, Completion plan, Track A,
items 6 (the binding) and 3 (Maven Central). It will be built as
`bindings/java`.

Every claim about this repository names the file it was read from. Every
claim about Java or Maven Central names its source, read on 2026-10-08.

## 1. Purpose and scope

The sixth binding, after Node, wasm, Dart, Python and the Rust façade
(`docs/QUESTIONS.md` Q3 gives the order). Java loads **the same shared
library** the C bundle, the Dart installer and the Python wheels carry
(`06-cicd/02-build-matrix.md`), through the Foreign Function and Memory
API (`java.lang.foreign`), which is in the JDK. So the package has no
runtime dependency, compiles nothing native on the consumer's machine, and
needs no build per JDK release.

As in the other bindings, everything mechanical is rendered from
`idl/api.json` by `cargo xtask gen ffi` and held to the boundary crates by
`cargo xtask check-ffi` (`xtask/src/ffi.rs`). The ergonomic layer, the
loader, the port adapter and the tests are written by hand, which ADR-0014
allows a binding to have in its own language.

It is not: a second computation of anything; a JNI binding; a Kotlin
surface (Kotlin calls a Java API as it is); or Android, which has no
`java.lang.foreign` and stays a v1.x question (roadmap, Track A item 6).

## 2. What was measured, and what was read

- **No struct crosses by value.** Every parameter in `idl/api.json` is a
  scalar, an enum or a pointer, and every return is a `Status`, a scalar
  or a pointer. (Read by tallying `functions[].params[].role` and
  `returns`.) A downcall therefore needs no ABI classification of
  aggregates, which is where FFM's platform differences live. The same
  was true of Teimeris's Java binding
  (`teimeris/docs/PLAN_JAVA.md` §9, Stage 1).
- **One field and no enum member collide with a Java keyword.** Enum
  members are spelled upper-case, as in Python, and no Java keyword is
  upper-case, so `ChartKind::Return` and `Choghadiya::Char` become
  `RETURN` and `CHAR` unharmed. The one collision is the field
  `CapabilitiesC.native`, read as a camel-case record component. Measured
  over `idl/api.json` against the Java SE keyword list; the rule in §4
  makes it a counted case rather than a found one.
- **The library is large to carry eight times.** The release library in
  this worktree (`target/release/libteistro_ffi.dylib`, darwin-arm64,
  built 2026-10-08) is 16.2 MB, and 4.7 MB deflated at level 9, which is
  roughly what a jar entry costs. A jar carrying the eight platforms of
  `xtask/src/platform.rs` would be of the order of 40 MB. §9 decides with
  this number, and step 0 of §13 measures it on the real artefacts.
- **The prior art is next door.** Teimeris built a Java binding on FFM
  (`teimeris/bindings/java`, `teimeris/docs/PLAN_JAVA.md`) against a
  hand-kept IDL: a named module, a fat jar with `native/<os>-<arch>/`,
  a loader that refuses a missing explicit path, `javac --release 22
  -Xlint:all -Werror`, a dependency-free test harness, and two consumer
  arms (Gradle and Maven). Where this page agrees with it, it says so;
  where it departs, it says why.

## 3. Why FFM, and why our own emitter

| option | why not, or why |
|---|---|
| JNI | hand-written or generated C glue compiled per platform; the Node addon is the measure of what a compiled bridge costs here (`bindings/node/native/src/generated.rs`, emitted by `crates/idl/src/emit/node.rs`). FFM needs no native code of ours at all |
| jextract | reads the C header, which carries C types and not the description's **roles** (`crates/idl/src/model.rs`, `Role`; `crates/idl/src/rules.rs`): it would emit bare method handles and leave the `struct_size` handshake, the owned strings, the blob descriptors and the error record to be wired by hand. Its output is platform-specific and meant to be generated once and committed (jextract's guide, [GUIDE.md](https://github.com/openjdk/jextract/blob/master/doc/GUIDE.md)), and it would sit outside `check-ffi` |
| JNA, JNR | a runtime dependency, and slower than FFM by design |
| **FFM, emitted from `idl/api.json`** | in the JDK since 22 (JEP 454); the generator is one more emitter beside `emit/python.rs` and `emit/dart.rs`, reading the same roles from `teistro_idl::rules`, gated by `check-ffi` byte for byte |

FFM's `Linker`, `SymbolLookup`, `Arena`, `MemorySegment` and
`MemoryLayout` are what `ctypes` and `dart:ffi` are to the other two, and
the closer analogue is Dart's: layouts are declared, fields are read
through handles, and the layout is *checked* at class initialisation
(a misaligned member layout is refused by `MemoryLayout.structLayout`),
which `ctypes` never does (`python-binding.md` §5).

### The floor: Java 22, tested on Java 25

FFM was final in **Java 22** (JEP 454). In Java 21 it was the third
preview (JEP 442): a class file compiled with `--enable-preview` runs
only on the release that compiled it, and the API changed between 21 and
22 (`MemorySegment.getUtf8String` became `getString`, `allocateUtf8String`
became `allocateFrom`, `Linker.Option.isTrivial` became `critical`). A 21
build would be a second binding in all but name.

The trade-off, stated: Java 21 LTS users, a large share of the JVM in
2026, cannot use the binding until they move to 25 LTS (September 2025).
Java 22 to 24 are non-LTS and out of free support. So the floor is the
API's, and the *audience* is 25; the tests run on 25 everywhere and on 22
on one row, so that "floor 22" is a run and not a compiler flag (§11).
A JNI fallback for 17 and 21 is rejected for the reason Teimeris gave
(`PLAN_JAVA.md` §8): two foreign-function paths double what every parity
gate compares.

## 4. What is generated and what is written by hand

Package `com.teispace.teistro` (§12 settles the namespace), a named
module of the same name.

| file or package | what it is | written by |
|---|---|---|
| `catalogue/*.java` in `com.teispace.teistro` | every enum of the description as a Java `enum` carrying `id()` and `key()`; a catalogue kind gains `UNKNOWN` and an `of(int)` that answers it for an id from a newer library, where a closed enum throws (the Python rule, `python-binding.md` §6) | the generator |
| `com.teispace.teistro.ffi.Native` | the raw layer: one `StructLayout` per boundary struct with explicit padding from `crates/idl/src/layout.rs`, a `VarHandle` per field, a `MethodHandle` per entry point built from its `FunctionDescriptor`, a `FunctionDescriptor` per callback, the constants, the generated size and offset table, and `GENERATED_ABI_VERSION` and `GENERATED_SDK_VERSION` | the generator |
| value types in `com.teispace.teistro` | a `record` per struct a binding shows (`rules::field_roles`): the handshake, padding, counts and presence flags absorbed exactly as Python and Dart absorb them; a flag is a `boolean`, a bit set an `EnumSet`, an enum array a `List` of members, an optional value an `Optional`/`OptionalDouble`; a builder for every request whose fields include optional ones | the generator |
| brands | `record Latitude(double degrees)` and the rest of the description's `api: brand=` quantities, each with a compact constructor that checks the stated range and names the field (ADR-0023; the architecture's Java row, `02-architecture/07-binding-architecture.md`) | the generator |
| `TeistroException` | unchecked, carrying the `Status`, the message, and the field, hint, range, key, detail and provider code of `ts_error` | the generator |
| blob decoders | one decoder per schema in `crates/ffi/src/schemas.rs`, reading `TSRB` (`crates/idl/src/blob.rs`) into typed views (§7) | the generator |
| records | the JSON records of `api.records` (`crates/idl/src/emit/records.rs`) as Java records, read by the binding's own JSON reader | the generator |
| `Messages` | the typed message and entity accessors, from `cargo xtask gen intl` (`crates/intl/src/generate.rs` gains a Java target, `xtask/src/intl.rs` its output path) | the generator |
| `Teistro`, `Context`, the areas | the layer a consumer uses (§6) | by hand |
| `NativeLibrary` | the loader and the build handshake (§8) | by hand |
| `Json` | a small JSON reader and writer: the JDK through 25 ships none, and a dependency for it would be the binding's first | by hand |
| `EphemerisProvider`, `HostProvider` | the port adapter over upcall stubs (§10) | by hand |
| `module-info.java` | exports `com.teispace.teistro` and `com.teispace.teistro.ffi` | by hand |
| `test/` | the surface, the decoders against `target/tsrb`, the layouts against the library, the loader's refusals, the provider | by hand |
| `example/*.java` | the shared examples (§11) | by hand |
| `parity/ParityRunner.java` | this binding's report for `check-parity` (not `Parity`, which the catalogue spells as an enum) | by hand |
| `typecheck/Wrong.java` | the usages that must not compile, each with the error it must raise | by hand |

The raw layer is **exported**, as Dart exposes `TeistroLibrary`
(`bindings/dart/lib/teistro.dart`): a consumer may call an entry point the
ergonomic layer does not wrap (the openness mandate). It is documented as
following the ABI, not the SDK's semver for the surface.

**Generated value types, against Teimeris's reversal.** Teimeris
generated none of its value types and hand-wrote them, because its
surface's types matched no C struct (`PLAN_JAVA.md` §9, "One decision
reversed while building"). Here the opposite is measured: Python and Dart
already generate a value class per boundary struct from one set of roles
(`emit/python.rs` `render_value_class`, `emit/dart.rs`
`render_value_class`), and the parity gate's `surface.*` keys compare the
areas, not the type names (`xtask/src/parity.rs`). Generating them is
what keeps Java's types the other bindings' types.

**Names.** `teistro_idl::emit::reserved` gains `JAVA` (the Java SE
keywords, with `true`, `false`, `null` and `_`), and the rule is Dart's,
because both are camel-case targets: a colliding name gains `Value`, so
`native` becomes `nativeValue`. Enum members are upper snake case, as
Java names constants. `cargo xtask surface` counts what the list catches,
as it does for the other three lists.

**Javadoc from the description.** Each item's documentation is the
description's, with unit, range and example appended by
`emit::field_doc` in prose style. Two things the other emitters never
needed: the text is HTML-escaped (`<`, `>`, `&`) and `*/` and a leading
`@` are neutralised, because `javadoc -Xdoclint:all` fails on raw markup
and the Central bundle needs a javadoc jar that builds (§12).

## 5. The layout, and how it is held

The C header asserts each struct's size at compile time; FFM computes
nothing and inserts no padding, so a layout is exactly what the emitter
wrote. Three checks hold it:

1. **At generation**, the emitter writes each struct's layout from
   `teistro_idl::layout::struct_layout` with `Target::LP64`, padding
   explicit, and `check-ffi` compares the file byte for byte.
2. **At class initialisation**, `structLayout` refuses a member that is
   not aligned, so a wrong padding is an exception before any call.
3. **At test time**, a generated test asserts `byteSize()` and every
   field's `byteOffset` against the generated table, on the machine the
   library was built on. That is stronger than Python's size-only
   assertion (`bindings/python/tests/test_sizes.py`), because FFM gives
   the offsets for free.

Only 64-bit targets exist in `xtask/src/platform.rs`, and the JDK ships
for no 32-bit target in that table, so the binding carries only the LP64
column and refuses at load an `ADDRESS` layout that is not 8 bytes,
naming what it found. `size_t` is read from
`Linker.nativeLinker().canonicalLayouts().get("size_t")` rather than
assumed (the canonical layouts are guaranteed for `size_t` and `bool`:
[Linker](https://docs.oracle.com/en/java/javase/22/docs/api/java.base/java/lang/foreign/Linker.html)).

**The `struct_size` handshake** is filled from the layout's `byteSize()`
in the generated marshalling, never from a literal, exactly as Python
fills it from `ctypes.sizeof` (`emit/python.rs`, `render_write`), so a
struct that gains a field keeps working against an older library. A test
proves a zero `struct_size` is refused with `SCHEMA_VERSION`
(`ffi-abi-and-api-description.md` §6), because a check that cannot fail
has been shipped before (`PLAN_JAVA.md` §7, risk 3).

**Scalars.** One Java carrier per scalar in `teistro_idl::model::Scalar`.
Java has no unsigned types, so an unsigned value reaching the surface is
widened (`u8` and `u16` to `int`, `u32` to `long`) and a `u64` stays a
`long` documented as unsigned; on the way in, a value outside the
scalar's range is refused by the generated marshalling and names the
field. In the raw layer they are the C widths, little-endian as every
target is.

## 6. The hand-written layer

```java
import com.teispace.teistro.*;

try (Teistro teistro = Teistro.open();
     Context sky = teistro.context(ContextOptions.builder()
             .ephemeris(Ephemeris.BUILTIN).build())) {
    Positions grid = sky.positions(PositionRequest.builder()
            .instants(2451545.0).bodies(Body.SUN).build());
    System.out.println(grid.at(0, 0).longitude());
}
```

- **`Teistro`** opens the library and checks its build (§8), and carries
  what needs no context: the versions, the default profile, `buildInfo()`
  (`ffi-abi-and-api-description.md`, "The build handshake").
- **`Context`** is `AutoCloseable`. `close()` frees it; a
  `java.lang.ref.Cleaner` is the backstop for a context nobody closed,
  holding the handle and not the object, so the cleaner can run.
  ADR-0007's finding 4 (results waiting for a collector exhaust memory)
  is why `try`-with-resources is the documented form and the cleaner only
  a safety net.
- **The areas** are the canonical ones of `surface-areas.md` §3, spelled
  as Java methods returning area objects: `calendar()`, `time()`,
  `intl()`, `keys()`, `frame()`, `chart()`, `almanac()`, `matching()`,
  `numerology()`, `engine()`, with `positions(...)`, `profile()`,
  `settings()`, `settingsJson()`, `settingsHash()` and `close()` at the
  root (§4 there). The parity runner prints the `surface.*` keys, so a
  Java grouping that drifts fails `check-parity` like any other
  (`xtask/src/parity.rs`, "Values, and shape").
- **Batch forms** as the others have them: `chart().foundMany(...)` takes
  a list and answers a list (`bindings/python/teistro/__init__.py`,
  `found_many`), so a grid is one request (the batch mandate).
- **Plugins and the engine** (ADR-0029, ADR-0030): `Plugin.load(path)`
  over `ts_provider_load`, a context built on it, and `engine().call(name,
  args)` with JSON both ways through `Json`, until an adapter generates
  the typed façade.

### Errors carry the record

A non-`OK` status raises `TeistroException`, built from the failing
call's own record: on a context method, `ts_context_last_error` read on
the same thread, under the context's lock, before anything else touches
the context; on a constructor, the `ts_error *out_error` the call wrote,
with `TS_ERROR_OWNED` set, read and then released with `ts_error_free`
(`ffi-abi-and-api-description.md` §6.1). A test makes a refusal after an
earlier refusal on the same context and checks the second exception
carries the second record. A brand out of range raises
`IllegalArgumentException` naming the field and the range; a blob of
another layout version or schema, or a library of another build, raises
`TeistroException` with `UNSUPPORTED`, as in Python (`python-binding.md`
§10).

### Threads, arenas and locks

The contract is one context, one thread at a time, movable between
threads (`ffi-abi-and-api-description.md` §3.1). Java is the first
binding with shared-memory threads in ordinary use, so the contract is
**enforced**, not only documented: each `Context` call holds the
context's `ReentrantLock`. Uncontended, that is a few nanoseconds;
contended, it turns a data race on the library's state into a wait. A
`ReentrantLock` rather than `synchronized`, because a virtual thread
blocked in `synchronized` pinned its carrier before Java 24 (JEP 491),
and the floor is 22.

Each call's inputs are marshalled in an `Arena.ofConfined()` opened and
closed inside the call. Teimeris chose the same and named the trade-off
(`PLAN_JAVA.md` §7, risk 2); it is measured in step 0 before anything
cleverer is tried. Upcall stubs and the provider's capability strings live
in an `Arena.ofShared()` owned by the context, because `close()` may run
on another thread than the one that made it.

`Linker.Option.critical` is not used in v1. It promises a function
neither runs long nor calls back into Java
([Linker.Option](https://docs.oracle.com/en/java/javase/22/docs/api/java.base/java/lang/foreign/Linker.Option.html)),
and any function that takes a context can reach a host provider; using it
anywhere waits for a measurement that says the downcall's cost matters.

## 7. The result blob

`ts_positions`, `ts_intl_render`, `ts_chart_found`, `ts_panchanga_days`
and `ts_naam_milan` answer with a `TSRB` blob the library allocated (the
`blob=` metadata in `idl/api.json`). The binding copies it **once into a
heap `byte[]`**, frees the library's copy immediately with `ts_blob_free`,
and decodes the copy through `MemorySegment.ofArray` with
`ValueLayout.JAVA_DOUBLE_UNALIGNED.withOrder(LITTLE_ENDIAN)` and its
siblings.

- **Heap, not off-heap.** A heap copy is memory the collector sees and
  is pressed by; an `Arena.ofAuto()` copy is off-heap memory the collector
  frees only when it happens to run, which is ADR-0007's finding 4 again.
- **Unaligned layouts**, because a heap segment over a `byte[]` promises
  only byte alignment; the columns are 8-aligned inside the blob
  (`crates/idl/src/blob.rs`), so the unaligned accessor costs nothing on
  x64 and arm64, and the aligned one would throw.
- **Little-endian named**, because the blob is little-endian by
  definition and a native-order layout would be right only by coincidence.

A column is a typed view (`DoubleColumn`, `IntColumn` …) with `get(i)`,
`size()` and `toArray()`. The decoder reads the layout version first and
refuses another, finds a section by id so an appended section is
skipped, and checks every offset against the length before it reads, as
the reference `Reader` does. One copy rather than none is Python's trade
(`python-binding.md` §7) for the same reason: a decoded result is safe to
keep.

## 8. Loading, identity and the build handshake

`Teistro.open()` looks, in order:

1. `-Dteistro.library=<path>`, then `$TEISTRO_LIBRARY` (the variable the
   other loaders read, `bindings/dart/lib/teistro.dart`). **An explicit
   path is binding**: a missing file throws and nothing else is tried.
   Teimeris found the fall-through loaded a library nobody asked for and
   reported a pass (`PLAN_JAVA.md` §9, Stage 4).
2. The packaged native for this platform, `native/<platform>/<file>`
   inside the jar, extracted (below).
3. The workspace's `target/release` and `target/debug`, as the Dart and
   Python loaders search them.
4. The platform loader by bare name (`SymbolLookup.libraryLookup(name,
   arena)`).

It then reads `ts_build_info` and applies the three rules every loader
applies (refuse another ABI or SDK version; refuse a sanitizer build
however found; refuse an unoptimised build it searched out rather than was
given), and records the path it loaded, for a harness to print.

**The platform name** is `xtask/src/platform.rs`'s, so one artefact name
serves every binding: `os.name` and `os.arch` give `linux`, `darwin`,
`win32` and `x64`, `arm64`, and a Linux host is `-musl` when
`/proc/self/maps` names a musl loader (`ld-musl-*`), which the musl rows
of verify prove. A host with no packaged native fails naming the platform
it computed and the places it looked.

**Extraction** is to a cache directory keyed by the library's SHA-256
(`${java.io.tmpdir}/teistro-<version>-<digest>/`, overridable with
`-Dteistro.cache`), written to a temporary name and moved atomically, and
checked against a digest recorded when the jar was staged, as the Dart
installer checks a download against `prebuilt.dart`
(`06-cicd/02-build-matrix.md`, "What is staged once"). Two departures from
Teimeris's `extractFromJar`, each for a defect it would have:
`deleteOnExit` cannot delete a loaded DLL on Windows, so a temp-file-per-run
scheme leaks one library per run there; and a content-addressed directory
lets concurrent JVMs share one copy without a race. A host whose temp
directory is mounted `noexec` sets `teistro.cache`, and the error says so.

**Native access.** Loading a library and making downcalls are restricted
methods. On 22 they warn without `--enable-native-access`; from JDK 24
the default mode is `warn` and `deny` is the stated future default
([JEP 472](https://openjdk.org/jeps/472)). The binding is a **named
module** so a consumer can grant it alone,
`--enable-native-access=com.teispace.teistro`, rather than
`ALL-UNNAMED`; Teimeris measured that the warning names the module,
telling a consumer exactly what to add (`teimeris/bindings/java/src/main/java/module-info.java`).
An executable jar may carry `Enable-Native-Access: ALL-UNNAMED`, the only
value that manifest attribute takes (JEP 472). The README says all of it.

## 9. Packaging

**The default coordinate is a jar carrying every platform**, as
Teimeris decided (`PLAN_JAVA.md` §4) and for its reason: Maven reads only
the POM, and a POM cannot choose an artefact by the consumer's platform.
JavaFX's POMs choose a classifier through OS-activated profiles, which
Gradle does not evaluate (hence JavaFX's Gradle plugin), and which select
the *build* machine's platform rather than the deployment's, so a jar
assembled on a Mac for a Linux container picks the wrong one. A jar that
carries all of them works for every JVM build tool with no configuration.

Its price is §2's number: of the order of 40 MB, where npm and pip hand
each consumer one platform. So two additions, decided now and built in
step 7:

- **Per-platform classifier jars** of the same artifact
  (`teistro-<version>-linux-x64.jar`), each the same classes and one
  native, for a consumer who wants the smaller image and names the
  platform. A *substitute* for the default, never beside it, so no
  classpath carries the classes twice.
- **The size is measured on what ships** (the deflated jar, per platform
  and in all), on the generated size page the roadmap's item 7 plans, and
  whether the release library should be stripped of symbols is decided by
  that measurement, for every binding at once, not by this one.

**Built by xtask, not by Gradle or Maven.** A jar is a zip, and
`xtask/src/wheel.rs` already writes a reproducible zip for the Python
wheel (sorted entries, one date, no owner) because setuptools builds only
for the host. The jar is the same problem: the stage job assembles every
platform's native into one artefact on one runner. So `cargo xtask package
stage` compiles the classes with the JDK's `javac --release 22`, runs
`javadoc`, and writes the main jar, the classifier jars, the sources jar,
the javadoc jar and the POM itself. The JDK's own tools are the
"target-language toolchain" ADR-0014 allows; Gradle or Maven *building*
the binding would be a second build of one thing and a plugin supply
chain to pin, and Teimeris's own build script says its artefact binds
nobody to its build tool (`teimeris/bindings/java/build.gradle.kts`).

The jar carries `LICENSE` and `NOTICE` under `META-INF/`, the description
at `META-INF/teistro/api.json` (the architecture's rule that the IDL ships
in every package, `02-architecture/07-binding-architecture.md`), the
digest table the loader checks, and `module-info.class`, so it is a
named module on the module path and needs no `Automatic-Module-Name`.
The POM has no dependencies.

The publish guard every package has (`06-cicd/03-release-process.md`, "One
version": private while the version is `0.0.0`) is a refusal in `package
stage` to write a Maven bundle at `0.0.0`.

## 10. A provider written in Java

`EphemerisProvider` is an abstract class with Dart's and Python's shape:
`name`, `bodies` and `positions` abstract, the rest defaulted, one call
per grid (`ffi-abi-and-api-description.md`, "A host-implemented
provider"). `HostProvider` binds one into `ProviderVtable` through
`Linker.upcallStub`, one stub per slot, in the context's shared arena so
the stubs live exactly as long as the vtable that points at them.

**An exception in an upcall kills the JVM.** "If the target method handle
does throw an exception, the JVM will terminate abruptly"
([Linker](https://docs.oracle.com/en/java/javase/22/docs/api/java.base/java/lang/foreign/Linker.html)).
So every stub's body catches `Throwable`, keeps it, and returns the port's
refusal code (`ProviderCode`, a described enum, so the binding writes no
number); the layer rethrows the provider's own exception on the caller's
side, with the library's refusal as a suppressed exception. This is
Python's trampoline rule (`python-binding.md` §9) with a harder failure:
Python's escaped exception returned zero, Java's ends the process. A test
binds a provider that throws and checks the process survives and the
caller catches the thrown object.

Coverage and validation are the SDK's, in `ask_positions`, so the adapter
checks neither (`ffi-abi-and-api-description.md`, the table there).

## 11. Tests, gates and CI rows

`cargo xtask check-java` (`xtask/src/java_binding.rs`), shaped like
`check-python` and skipped with a note when no JDK is present (ADR-0014):

1. build the library and the blob fixtures (`binding::library`,
   `binding::blob_fixtures`);
2. compile the binding, the tests and the examples with `javac --release
   22 -Xlint:all -Werror`, which makes a restricted call that is not
   marked, or an API above 22, a failure;
3. run the tests on a plain harness of our own (a few dozen lines), with
   `TEISTRO_LIBRARY` and `TEISTRO_FIXTURES` set, as Python runs
   `unittest`: no JUnit, so the gate fetches nothing and pins nothing,
   which is also Teimeris's choice (`PLAN_JAVA.md` §9, Stage 4);
4. the shared examples, through `examples.rs` gaining `Binding::Java`;
5. `typecheck/Wrong.java` compiled alone: every `// expect:` line must be
   reported and no error unexpected (`xtask/src/dart_binding.rs`,
   `wrong_usages`), the Java half of "a swapped latitude and longitude
   does not compile";
6. `javadoc -Xdoclint:all -Werror` over the exported packages.

Java is run through **one command builder** that sets
`-Dstdout.encoding=UTF-8 -Dstderr.encoding=UTF-8` and
`--enable-native-access=com.teispace.teistro`, for the defect
`binding::python_command` records: the Windows console's code page
cannot print `सोमबार`, and a gate that builds its own command forgets.
A lint holds it, as `python-runs-in-utf8-mode` holds Python's. Every
number the runner and the examples print is formatted with `Locale.ROOT`,
because a comma-decimal locale prints `280,37` (`PLAN_JAVA.md` §9,
Stage 6).

The tests cover every entry point through the ergonomic layer; the
decoders against `target/tsrb`; the layout table; the catalogue's round
trips and `UNKNOWN`; the loader's refusals (a missing explicit path, a
wrong build, a missing platform); a provider answering a real grid and one
that throws; and the quality bar's leak check, 10,000 create-use-drop
cycles with a flat RSS curve, which `05-testing/01-quality-bar.md` already
names for Java.

**Parity.** `bindings/java/parity/Parity.java` walks the scenario of
`bindings/python/parity.py` and prints the same `key<TAB>value` lines;
`xtask/src/parity.rs` gains it as one more runner, compared against the
first report like the rest. The shared examples (`bindings/*/example/`)
gain a Java set of the same names, compared line for line. That is the
acceptance test reused rather than rewritten, as the wasm package passed
Node's own suite.

**CI.** In `.github/workflows/verify.yml`'s `bindings` matrix, every row
gains `actions/setup-java` (Temurin 25) and a `check-java` step before
`check-parity`. The `linux-x64` row also installs JDK 22 and runs the
tests on it, so the floor is run, not only compiled. The musl rows run
inside Alpine (`xtask/alpine.sh`), whose own OpenJDK packages are measured
in step 0 before the row is promised; a row without a JDK is excused at
the point it is made, as Dart is on musl
(`06-cicd/02-build-matrix.md`), not left to skip. Windows arm64's JDK
distribution is measured the same way. `check-package` gains a Maven
consumer arm: the staged bundle unpacked into a file repository, an empty
Maven project depending on the coordinate, and `packaging/Consumer.java`
asserting the four facts the C smoke test prints, with no
`TEISTRO_LIBRARY` to help it. Gradle reads the same POM, and the binding
publishes no Gradle module metadata, so Teimeris's reason for a second
arm (a `.module` file Maven never sees, `PLAN_JAVA.md` §7, risk 7) does
not arise; a Gradle arm is an open question (§14), not a requirement.

## 12. Publishing to Maven Central

**The Central Publisher Portal is the only route.** OSSRH reached end of
life on 2025-06-30 ([announcement](https://central.sonatype.org/news/20250326_ossrh_sunset/)).

**The namespace is `com.teispace`**, verified once by a DNS TXT record on
`teispace.com`, the domain `SECURITY.md` and `CODE_OF_CONDUCT.md` already
give; a verified namespace covers every group beneath it
([namespace](https://central.sonatype.org/register/namespace/)). The
coordinate is `com.teispace:teistro`. Teimeris chose `io.teispace`
(`teimeris/bindings/java/build.gradle.kts`) and has not published; one
Teispace namespace across the two projects is worth asking for (§14).

**What a release uploads** is one bundle in Maven repository layout,
`com/teispace/teistro/<version>/`, holding the POM, the main jar, the
classifier jars, the `-sources.jar` and `-javadoc.jar`, an `.asc` PGP
signature for each, and `.md5` and `.sha1` for each (`.sha256` and
`.sha512` optional); the POM carries name, description, url, the
Apache-2.0 licence (`Cargo.toml`), developers and scm
([requirements](https://central.sonatype.org/publish/requirements/),
[bundle](https://central.sonatype.org/publish/publish-portal-upload/)).
`package stage` writes all of it but the signatures, so the bundle is
reproducible and checked by `check-package` before anything is signed.

**How it is sent.** `POST
https://central.sonatype.com/api/v1/publisher/upload?publishingType=…` with
the bundle as the multipart part `bundle`, authenticated by `Authorization:
Bearer <base64(token-username:token-password)>`, then `POST
/api/v1/publisher/status?id=<deployment>` polled until `PUBLISHED` or
`FAILED` ([Publisher API](https://central.sonatype.org/publish/publish-portal-api/)).
The first release uses `USER_MANAGED`, so the validated deployment is
inspected in the Portal and published by hand; later releases use
`AUTOMATIC`. The upload is `cargo xtask publish maven` (one HTTP client in
xtask), not a Maven plugin, so the publish runs the same on a laptop.

**Credentials.** Central has no OIDC trusted publishing that this search
found (2026-10-08), so this is the one registry that takes secrets: a
Portal user token and the PGP key with its passphrase, held in the
`release` environment (`06-cicd/03-release-process.md`), the token minted
for a release and revoked after it, which the release process's rule
allows ("a token minted, used once and revoked within the hour"). The PGP
key is an identity and cannot be short-lived; its public half is sent to
`keys.openpgp.org` and `keyserver.ubuntu.com`, and its fingerprint is
written in `SECURITY.md`. Signing uses `gpg --batch --detach-sign
--armor`, which the runner has.

**Provenance.** The jars and the bundle go through the existing
`actions/attest-build-provenance` step and onto the GitHub release with
the other archives. Central also validates Sigstore bundles
(`.sigstore.json`) when present, optional beside PGP, which remains
required ([Sonatype](https://central.sonatype.org/news/20250128_sigstore_signature_validation_via_portal/)).
Keyless signing from the publish job, which already has `id-token: write`,
is step 8's second half.

**Order in `release.yml`'s `publish` job**: after the GitHub release,
alongside npm, pub.dev and PyPI; a failed Central deployment fails the
job, and nothing published elsewhere is withdrawn (there is no
un-publish, `03-release-process.md`, "Withdrawing").

## 13. Order of work

0. **Measure, no code kept.** A JDK 22 and a JDK 25 on each verify row
   (Temurin's Windows arm64 and Alpine builds especially); FFM's warning
   behaviour without the flag on 22 and on 25; the cost of a downcall with
   a confined arena per call against Python's `ctypes` on `ts_key_parse`
   and on a positions grid; the deflated size of each platform's library
   from a release run's `target/dist`.
1. **The emitter.** `crates/idl/src/emit/java.rs` and
   `reserved::JAVA`; `xtask/src/ffi.rs` writes its files and prunes
   strays under `bindings/java/src/main/java/com/teispace/teistro/` as it
   prunes the reference; `check-ffi` holds them. Constants, enums,
   layouts, handles, value records, brands, the exception, the decoders.
   **Begun 2026-10-08:** the constants, the enums with `Member` and
   `Catalogued`, the exception, and `Native`: each struct's layout with
   its padding, its size, alignment and offsets, a `VarHandle` per value
   field, a descriptor per callback and a method handle per entry point.
   The generated tree is its own source root, `bindings/java/generated`,
   so pruning can never reach a hand-written file; the hand-written layer
   is `bindings/java/src`. A struct's class is named as the boundary crate
   names it (`Native.TsString`), because the binding's name for it can be
   a JDK type's (`String`). **Then:** a record per struct a binding
   shows, writing itself into its C struct and reading itself back, with
   an unsigned field carried one size wider and refused outside its C
   range by the name the caller wrote; a record per brand, checking its
   stated range; and `Calls`, package-private, a static method per entry
   point marshalled from the parameter roles as Python's are, with a
   confined arena only where the call allocates. **Then** the decoders,
   in `com.teispace.teistro.blob` so a section's name never meets an
   enum's: a class per schema, a record per shared shape, and a class
   per column section whose getters read a row from the copy `Calls`
   took of the library's bytes, every offset checked before it is read.
2. **The first slice, end to end.** `NativeLibrary` from
   `TEISTRO_LIBRARY` and `target/release` only, the handshake,
   `Teistro`, `Context` with `calendar().convert`, `time().resolve`,
   `intl().render` and `positions`, enough to print the C smoke test's four
   facts; one refusal raised with its field and hint, and one after
   another; `check-java` steps 1 to 3 and 5. This proves the generated
   layer, the struct handshake, an owned string, a blob, the error record
   and the build refusal in one program, and is the smallest thing that
   does.
   **Begun 2026-10-08:** `Teistro.open()` with the property, the variable
   and the workspace search and the three build rules; `Context` with its
   lock, its cleaner, a constructor's owned record read and freed, and a
   method's record read under the lock; `profile`, `settingsJson`,
   `settingsHash`, `keyId` and `keyName`; `Json`, a strict reader. The
   tests, with no framework, run as `check-java` in verify on every row
   but the two musl ones: the struct sizes, a refusal after a refusal, a
   refused constructor, a closed context. **Then:** the calendar, time,
   keys, frame and intl areas over `Calls`, each call under the context's
   lock, and the library's own calls on `Teistro`; tested by a round trip
   through Bikram Sambat, a scale conversion and back, a frame packed and
   unpacked, and a month of 300 refused as `` `month` `` before it is
   sent. **Then** `positions` and `intl().render` over the decoders,
   with a JSON writer beside the reader; a blob cut short, of another
   magic, version, schema or length is refused rather than misread.
3. **The areas**, to `surface-areas.md`'s table, and `Parity.java`;
   `check-parity` gains Java.
   **Begun 2026-10-08:** a founded chart's every reading, Vedic and
   Western, as a typed record read from its batch's sections and parsed
   once per batch; the almanac with its muhurta, festival, year and
   eclipse sections; matching, numerology, rashifal and the engine's own
   functions. Tests run each against the real library: a chart asked for
   every reading at once, the day the almanac shares, the Chaldean order
   of the dignities, KP under its own ayanamsha, and the areas beside the
   chart. A registered dasha system reads back by its key, from the
   names the context registered. **Built 2026-10-08:**
   `parity/ParityRunner.java` prints every key the other runners print,
   and `check-parity` holds Java to Node value for value; under
   `TEISTRO_STRICT` a machine without a JDK fails the gate rather than
   skipping it.
4. **The shared examples**, `Binding::Java` in `xtask/src/examples.rs`.
   **Built 2026-10-08:** `bindings/java/example` holds a class per shared
   example, named in Java's case and read back as the others' file name,
   compiled from the unnamed module at release 22 with every lint an
   error, so an example reaches only what the module exports.
   `check-java` runs them and `check-parity` compares them line for line
   with the other four sets. `Civil` gives Java Python's `date`, `at` and `iana_zone`, and
   `positions` answers a `PositionGrid` read cell by cell, as Python's
   does. The module's sources reach `javac` through an argument file:
   passed one by one they outgrew a Windows command line.
5. **The provider, plugins and the engine**, with the throwing-provider
   test and the leak check.
   **Begun 2026-10-08:** `EphemerisProvider`, Python's contract with
   its defaults, bound by `HostProvider`: two upcall stubs and the
   capabilities described once, in a shared arena the context closes
   after it frees the handle. Every upcall catches everything, because an
   exception that escapes an FFM upcall ends the JVM. What the provider
   threw is thrown on the caller's side from `Context.locked`, so every
   call a provider answers carries it, not `positions` alone: an
   unchecked exception as itself, a checked one as a
   `ProviderException`'s cause, with the library's refusal suppressed
   beside it. A column of the wrong length is refused, never padded.
   The tests found a chart through a Java provider, take a throw back as
   the same object, then a later refusal as the library's own, and bind
   and release two hundred providers. **Then** plugins and the
   overrides beyond `positions`.
6. **Messages and records**: the Java target of `gen intl`, the
   generated records over `Json`. **Begun 2026-10-08:** `gen intl`
   writes `com.teispace.teistro.messages.Messages` into its own source
   root, `bindings/java/messages`, with an accessor per message and
   entity, and `IntlArea.messages()` and `entity(key)` read through it.
7. **Packaging**: the jar with every native, the classifier jars, the
   digest table, the cache extraction, the POM and the bundle from
   `package stage`; the Maven consumer arm in `check-package`; the jar's
   platform list held to `PLATFORMS`, so a missing native fails at staging
   and not on a machine we do not have (`PLAN_JAVA.md` §10).
8. **Central**: the namespace verified, the key published, `cargo xtask
   publish maven`, a `USER_MANAGED` dry run from a dispatch that uploads
   and does not publish; then Sigstore bundles.
9. **The Teimeris adapter's Java package** (`adapters/ephemeris-teimeris/`
   gains `java`), type-checked by `check-java` as the others are by their
   binding's gate.
10. **The pages**: `bindings/java/README.md` with the native-access flag,
    the Windows encoding note and the coordinates for Maven, Gradle and
    sbt (only Maven gated, and said so); the site's guides; the build
    matrix and release-process pages' rows; `binding-surface-measured.md`
    swept for its per-target counts.

## 14. Open questions

- **The namespace across Teispace: decided 2026-10-08, `com.teispace`.**
  Central verifies a namespace by a DNS record on the domain it reverses,
  and Teispace's domain is `teispace.com` (`SECURITY.md`'s address), so
  `io.teispace` would need a domain Teispace does not use. Teimeris's
  plan moves to `com.teispace.teimeris` before it publishes.
- **The fat jar's size.** Step 0 and step 7 measure it; if the default
  coordinate passes a size a consumer would refuse, the classifier jars
  become the documented default for container images and the fat jar
  stays for "it just works".
- **A Gradle consumer arm.** Not needed for the POM's sake (§11); worth
  having if Gradle's variant selection ever reads something the binding
  publishes.
- **A pool of contexts.** The ABI page says "the bindings' pools give
  every worker its own" (`ffi-abi-and-api-description.md` §3.1), and no
  binding in `bindings/` has one yet. Java, with real threads, is where a
  `ContextPool` earns its place first; designed with the others, not here
  alone.
- **`critical` downcalls** for the context-free entry points, if step 0
  finds the downcall's own cost visible.
- **Android** (v1.x): no FFM there; a JNI layer generated from the same
  description, or the Dart package's Flutter route, decided when someone
  asks.
