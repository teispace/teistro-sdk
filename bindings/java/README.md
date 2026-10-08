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
`java --enable-native-access=com.teispace.teistro …`.

## Layout

- `generated/`: written by `cargo xtask gen ffi` from `idl/api.json` and
  held by `cargo xtask check-ffi`. Do not edit it.
- `src/`: the hand-written layer: the loader, `Teistro`, `Context`, the
  JSON reader.
- `test/`: the binding's tests, run by `cargo xtask check-java`.

`Teistro.open()` loads the library named by `-Dteistro.library` or
`TEISTRO_LIBRARY`, and nothing else when one is named; otherwise the
workspace's `target/release`, then `target/debug`.
