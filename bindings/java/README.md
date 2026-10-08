# Teistro for Java

The Teistro SDK for Java 22 and later: the library's C boundary through
the Foreign Function and Memory API, with no native code of its own and
no dependency. **In progress**; the design is
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
