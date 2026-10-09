// A two-group study, as `03-design/research.md` asks one to be run: the
// predicates named, the labels and the test fixed, and the input hash
// published before any data are read. `cargo xtask check-dart` runs this
// file, and every binding's `research` prints these lines.

import 'package:teistro/teistro.dart';

void main() {
  final teistro = Teistro.open();
  final ctx = teistro.context(
    profile: 'nepali-default',
    ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
  );

  // Forty-eight births at Kathmandu, two months and an hour apart, and the
  // first sixteen called the cases. The labels mean nothing, so a study that
  // reads them honestly finds nothing: that is what the corrections are for.
  final place = Observer(
    latitudeDeg: Latitude(27.7172),
    longitudeDeg: Longitude(85.324),
    altitudeM: Altitude(1400),
  );
  final births = [
    for (var i = 0; i < 48; i++)
      ResearchBirth(
        instant: 2444240.5 + 61.37 * i + (i % 24) / 24,
        place: place,
        utcOffsetSeconds: 20700,
      ),
  ];
  const rules = RuleRequest(shipped: [ShippedRules.yogas]);
  final design = ResearchDesign(
    groups: [for (var i = 0; i < 48; i++) i < 16 ? 1 : 0],
  );

  final table = ctx.research.counts(
    births: births,
    rules: rules,
    design: design,
  );
  print('${table.rows.length} rules over ${births.length} births');

  final tested = ctx.research.compare(
    births: births,
    rules: rules,
    design: design,
    test: ResearchGroupTest(
      seed: BigInt.from(2026),
      permutations: 999,
      contrast: const ResearchContrast.caseVsRest(1),
      alpha: 0.05,
    ),
  );
  // The registration: the births, the rules, the labels and the test.
  print('registered as ${tested.provenance.inputHash}');
  print(
    '${tested.permutations} permutations, none can say less than '
    'p = ${tested.resolution.toStringAsFixed(4)}',
  );

  int under(bool Function(ResearchUnderAlpha) method) =>
      tested.rows
          .where((row) => row.underAlpha != null && method(row.underAlpha!))
          .length;
  print(
    'under 0.05: ${under((u) => u.raw)} raw, ${under((u) => u.maxT)} after '
    'max-T, ${under((u) => u.holm)} after Holm',
  );

  // The smallest raw p, the first such row on a tie, and what the family
  // makes of it.
  final best = tested.rows.reduce(
    (kept, row) => row.p.value < kept.p.value ? row : kept,
  );
  final [rest, cases] = best.counts;
  print(
    '${best.predicate}: ${cases.present} of ${cases.present + cases.absent} cases, '
    '${rest.present} of ${rest.present + rest.absent} others, '
    'p ${best.p.value.toStringAsFixed(3)}, max-T ${best.adjusted.maxT.toStringAsFixed(3)}',
  );
  final effect = best.effect;
  if (effect != null) {
    final d = effect.riskDifference;
    print(
      'difference ${d.estimate.toStringAsFixed(3)} '
      '(${d.low.toStringAsFixed(3)} to ${d.high.toStringAsFixed(3)})',
    );
  }

  ctx.dispose();
}
