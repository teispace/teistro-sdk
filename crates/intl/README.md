# teistro-intl

Teistro Intl, the SDK's localisation layer
(`docs/03-design/intl-engine-and-packs.md`, ADR-0010): one standard for
every string the SDK renders and for the applications that adopt it.

- **Sources** under `i18n/<locale>/`: `_meta.json` (direction, numbering
  system, grouping, fallback chain, contexts, list patterns) and one JSON
  file per namespace; `en-Latn` is the base locale, the source of every
  key and parameter; `sdk.entity` holds entity records whose keys are the
  SDK's catalogue keys, every other namespace holds messages.
- **The grammar** is the stable `MessageFormat 2` (Unicode LDML 47) in
  full, with a fixed function set bound to the SDK's types: `:string`,
  `:integer`, `:number`, `:dms`, `:zodiac`, `:entity`, `:list`, `:msg`.
  Plural rules and locale parsing come from ICU4X.
- **The engine** (`Intl`) renders a key with typed parameters under an
  explicit locale and its declared fallback chain, says which locale
  answered, and never renders a blank: the worst case is the key itself.
- **Validation** (`validate`) gates a tree before it is built: syntax with
  offsets, parity with the base, selectors against the locale's plural
  categories and contexts, references, and the catalogue as the authority
  for every entity key and kind; coverage per locale and per catalogue
  kind is reported.
- **Packs** (`pack`): the `.tpack` container, one locale and namespace,
  and the `.tbundle`, one locale with its metadata once; zero-copy reads
  behind a checksum, a SHA-256 for the provenance envelope.
- **Typed accessors** (`generate`): TypeScript, Dart and Rust surfaces
  holding keys and parameter shapes only, never text; the SDK's own
  namespaces are generated into `messages` by `cargo xtask gen intl` and
  held by `check-intl`.
- **The command line** (`teistro-intl`, `cli`): `validate`, `build`,
  `gen`, `render`, `extract`, `report`; every command a library function.

```rust
use teistro_core::catalogue::Graha;
use teistro_intl::messages::sdk::reason::GrahaInBhava;
use teistro_intl::source::Tree;
use teistro_intl::{Intl, Value, params, sdk_root};

let tree = Tree::load(&sdk_root()).expect("the SDK's sources");
let mut intl = Intl::from_tree(&tree).expect("plural rules for every locale");
intl.set_locale("ne-Deva-NP").expect("a shipped locale");

// Typed: the key and the parameters are checked by the compiler.
let typed = intl.render_typed(&GrahaInBhava { bhava: 7, graha: Graha::Jupiter });
// Or by key, for tools and runtime overrides.
let by_key = intl.render(
    "sdk.reason.grahaInBhava",
    &params([("graha", Value::catalogued(Graha::Jupiter)), ("bhava", Value::Int(7))]),
);
assert_eq!(typed, by_key);
assert_eq!(typed.resolved_from.as_deref(), Some("ne-Deva-NP"));
```

```sh
cargo run -p teistro-intl -- validate
cargo run -p teistro-intl -- render --locale ne-Deva-NP sdk.reason.grahaInBhava --param graha=@graha.JUPITER --param bhava=7
cargo run -p teistro-intl -- build --bundle --out target/packs
cargo run -p teistro-intl -- gen --target ts,dart,rs --out target/intl-gen
cargo run -p teistro-intl -- extract --locale ta-Taml-IN
cargo xtask check-intl
```

The `cli` feature (on by default) brings the command line; a binding that
embeds the engine turns it off.
