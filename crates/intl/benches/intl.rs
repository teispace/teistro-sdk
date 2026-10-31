//! The budgets of `docs/03-design/intl-engine-and-packs.md` §10: a render
//! under 5 µs, a pack lookup under 1 µs, a pack verified under 5 ms per ten
//! thousand entries, a locale loaded from packs under 1 ms per thousand;
//! and the lexicon's (`docs/03-design/lexicon.md` §5): built once, then a
//! lookup or a date read per call.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    missing_docs,
    reason = "a benchmark stops on a broken fixture; the harness macro's items are undocumented"
)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use teistro_core::catalogue::Graha;
use teistro_intl::lexicon::LookupRequest;
use teistro_intl::pack::{self, Pack, locales_from_packs};
use teistro_intl::source::Tree;
use teistro_intl::{Intl, Value, params, sdk_root};

fn benches(c: &mut Criterion) {
    let tree = Tree::load(&sdk_root()).expect("the SDK's sources");
    let mut intl = Intl::from_tree(&tree).expect("plural rules");
    intl.set_locale("ne-Deva-NP").expect("a shipped locale");
    let with_entity = params([
        ("graha", Value::catalogued(Graha::Jupiter)),
        ("bhava", Value::Int(7)),
    ]);
    c.bench_function("render: a literal", |b| {
        b.iter(|| intl.render(black_box("sdk.reason.appName"), &params([])));
    });
    c.bench_function("render: an ordinal select with an entity", |b| {
        b.iter(|| intl.render(black_box("sdk.reason.grahaInBhava"), &with_entity));
    });
    let at = params([
        ("graha", Value::catalogued(Graha::Mars)),
        ("longitude", Value::Num(222.5763)),
    ]);
    c.bench_function("render: an entity and a zodiac angle", |b| {
        b.iter(|| intl.render(black_box("sdk.reason.grahaAt"), &at));
    });
    let source = "\
.input {$count :integer} .input {$graha :entity kind=graha form=prose} .match $count \
0 {{No planet conjoins {$graha}}} one {{One planet conjoins {$graha}}} * {{{$count} planets conjoin {$graha}}}";
    c.bench_function(
        "parse: a matcher with two declarations and three variants",
        |b| {
            b.iter(|| teistro_intl::mf2::parse(black_box(source)));
        },
    );
    let nepali = tree.locales.get("ne-Deva-NP").expect("the Nepali locale");
    c.bench_function("pack: build the Nepali entity namespace", |b| {
        b.iter(|| pack::build(black_box(nepali), "sdk.entity"));
    });
    let bytes = pack::build(nepali, "sdk.entity").expect("a pack");
    c.bench_function("pack: parse and verify it", |b| {
        b.iter(|| Pack::parse(black_box(&bytes)));
    });
    let parsed = Pack::parse(&bytes).expect("a pack");
    c.bench_function("pack: look up graha.SUN", |b| {
        b.iter(|| parsed.get(black_box("graha.SUN")));
    });
    let bundle = pack::build_bundle(nepali).expect("a bundle");
    c.bench_function("bundle: parse and verify the Nepali locale", |b| {
        b.iter(|| pack::Bundle::parse(black_box(&bundle)));
    });
    let mut packs: Vec<Vec<u8>> = Vec::new();
    for locale in tree.locales.values() {
        for name in locale.namespaces.keys() {
            packs.push(pack::build(locale, name).expect("a pack"));
        }
    }
    let borrowed: Vec<&[u8]> = packs.iter().map(Vec::as_slice).collect();
    c.bench_function(
        "engine: build from every pack, plural rules included",
        |b| {
            b.iter(|| Intl::new(locales_from_packs(black_box(&borrowed)).expect("locales")));
        },
    );
    lexicon(c, &tree);
}

/// The lexicon: built from nothing (what the first lookup on a context
/// pays), then a sentence looked up on a built one.
fn lexicon(c: &mut Criterion, tree: &Tree) {
    let sentence = LookupRequest::from_json(
        r#"{"text": "आज एकादशी हो, चन्द्रमा रोहिणीमा छ र सोमबार हो। Today is Ekadashi and the Moon is in Rohini."}"#,
    )
    .expect("a request");
    c.bench_function("lexicon: build every locale's, then look up", |b| {
        b.iter_batched(
            || Intl::from_tree(tree).expect("plural rules"),
            |fresh| fresh.lookup(black_box(&sentence)).expect("an answer"),
            criterion::BatchSize::LargeInput,
        );
    });
    let built = Intl::from_tree(tree).expect("plural rules");
    built.lookup(&sentence).expect("an answer");
    c.bench_function("lexicon: look up two sentences of 18 words", |b| {
        b.iter(|| built.lookup(black_box(&sentence)));
    });
}

criterion_group!(intl, benches);
criterion_main!(intl);
