# Teistro for Java

The Teistro SDK for Java 22 and later: the library's C boundary through
the Foreign Function and Memory API, with no native code of its own and
no dependency. The design is
[`docs/03-design/java-binding.md`](../../docs/03-design/java-binding.md).

```java
import com.teispace.teistro.*;

try (Teistro teistro = Teistro.open();
     Context sky = teistro.context(ContextOptions.builder().profile("nepali-default").build())) {
    System.out.println(sky.settingsHash());
}
```

Run with the module granted native access:
`java --enable-native-access=com.teispace.teistro …` on the module path,
or `--enable-native-access=ALL-UNNAMED` on the class path.

## Installing

The package is `com.teispace:teistro` on Maven Central. Its default jar
carries the library for every platform a release builds; a classifier jar
(`linux-x64`, `linux-arm64`, `linux-x64-musl`, `linux-arm64-musl`,
`darwin-x64`, `darwin-arm64`, `win32-x64`, `win32-arm64`) carries one, with
the same classes, for an application that ships to one platform.

```xml
<dependency>
  <groupId>com.teispace</groupId>
  <artifactId>teistro</artifactId>
  <version>VERSION</version>
  <!-- one platform only: <classifier>linux-x64</classifier> -->
</dependency>
```

```kotlin
implementation("com.teispace:teistro:VERSION")
```

```scala
libraryDependencies += "com.teispace" % "teistro" % "VERSION"
```

Maven is the one the release checks: `cargo xtask check-package` resolves
the staged repository with it, both jars, and refuses a wrong checksum.
Gradle and sbt read the same POM.

On first use the jar's library for the host is written to a cache,
`teistro-<version>-<digest>` under `java.io.tmpdir`: a directory only its
owner may write (one that is anyone else's, or group- or world-writable,
is refused), the file checked against the digest compiled into the jar
and checked again each time it is reused. Three system properties change what is found:

- `teistro.library`: a library of your own, and nothing else is tried
  (`TEISTRO_LIBRARY` likewise);
- `teistro.cache`: where the jar's library is written, for a temporary
  directory that is shared or on a `noexec` mount;
- `teistro.platform`: the platform name to load, for a host the jar's
  detection reads wrongly.

## Using it

Runnable programs live in [`example/`](example/), and
`cargo xtask check-java` compiles and runs every one, so none of them can
drift from what the binding does; `cargo xtask check-parity` holds each
to the other bindings' program of the same name, line for line. They are
meant to be read in order, from a quickstart to an ephemeris of your own,
and [`example/README.md`](example/README.md) says what each is really
teaching and how to run one.

The one thing to know before writing anything real is in
[`example/BirthChart.java`](example/BirthChart.java): **the canonical
frame is tropical**, because that is what an ephemeris computes. A Vedic
chart asks for a sidereal one and the SDK completes it, naming every step
it applied.

**Print with `-Dstdout.encoding=UTF-8`** whenever you print what this SDK
returns. `System.out` writes the console's encoding, and on Windows that
is a code page with neither Devanagari nor the degree sign, so each prints
as `?`; the string itself was never the problem. `check-java` sets it, so
the examples run there as they do anywhere.

## Types

A quantity that would otherwise be swappable is its own record, checked
when it is made:

```java
Observer place = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
```

Passing a `Latitude` where a `Longitude` is wanted is a compile error,
and a value outside the range the description states is an
`IllegalArgumentException` naming it. A catalogue member is an enum with
its key, a value a day may not have is an empty `Optional`, never a
sentinel, and a refusal is a `TeistroException` carrying its `status()`,
the `field()` it names and a `hint()`.

## Which ephemeris

A context with no ephemeris computes calendars, times and messages;
positions need one. `ephemeris` names it, or names an **ordered chain**
tried in order (ADR-0029):

```java
import com.teispace.teistro.teimeris.Teimeris;

// A real engine, and the SDK's own only if it is not there.
ContextOptions.builder()
    .ephemeris(Teimeris.builder().dataDir("./ephe").build(),
               EphemerisChoice.of(Ephemeris.BUILTIN))
    .build();
```

**That is the intended path.** In most cases a consumer should be on a
real engine, such as Teimeris or Swiss Ephemeris, installed as its own
package under its own licence (the Teimeris adapter's is
`com.teispace.teistro.teimeris`), and `Ephemeris.BUILTIN` is the fallback
that makes a chart compute with nothing else installed. `Ephemeris.TEST`
selects the analytic test provider, whose positions are **not
astronomy**.

A chain is a caller *saying* they will accept the fallback: a context
asked for an engine and given the built-in without being told is the
silence this refuses, and nothing in the chain opening is one refusal
naming each entry that failed. An engine brings its own operations with
it at `context.ephemeris()`, and the adapter's package carries a typed
façade over them.

## An ephemeris of your own

Extend `EphemerisProvider` and give the context one with
`ContextOptions.builder().provider(…)`. It is asked once for a whole grid,
never in a loop, and everything but the name, the bodies and the
positions has a default; [`example/YourOwnEphemeris.java`](example/YourOwnEphemeris.java)
is the contract in full.

What the provider throws reaches the caller as itself, or as a
`ProviderException`'s cause when it is checked. Only a code crosses the C
boundary, so the binding keeps the exception and throws it again on the
other side. That matters more here than in any other binding: an
exception that escaped an FFM upcall would end the JVM, so every upcall
catches everything.

## Layout

- `generated/`: written by `cargo xtask gen ffi` from `idl/api.json` and
  held by `cargo xtask check-ffi`. Do not edit it.
- `src/`: the hand-written layer: the loader, `Teistro`, `Context`, the
  JSON reader.
- `test/`: the binding's tests, run by `cargo xtask check-java`.
- `packaging/`: the consumer `cargo xtask check-package` runs against the
  staged jar, knowing nothing but the published names.

`Teistro.open()` loads the library named by `-Dteistro.library` or
`TEISTRO_LIBRARY`, and nothing else when one is named; otherwise the
jar's own library for the host; then the workspace's `target/release`,
then `target/debug`; then the platform loader's search path.

## Running the tests

`cargo xtask check-java` does all of it: it compiles the module, its
tests and the examples at release 22 with every lint an error, runs the
tests against the real library, and runs every example. It needs a JDK 22
or later on the path.
