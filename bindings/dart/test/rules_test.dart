// Every rule the SDK ships, and every request record the SDK's deep
// examples fill to the last optional field, built with the generated
// request classes and held to the JSON they came from: the JSON the classes
// write must be the source's, and two builds of one value must be one
// value. The rules use most of the condition language, so a variant whose
// class writes a field under the wrong name, or forgets one, fails here.
//
// The layer has no JSON reader for requests (a consumer builds them in
// code), so the test reads each rule into the classes by reflection: a
// field by its constructor parameter, a variant by its `type`.

import 'dart:convert';
import 'dart:io';
import 'dart:mirrors';

import 'package:teistro/teistro.dart';
import 'package:test/test.dart';

final LibraryMirror _layer =
    reflectClass(RuleDefinition).owner! as LibraryMirror;

final List<ClassMirror> _classes =
    _layer.declarations.values.whereType<ClassMirror>().toList();

/// `planet-in-house` as a class name's tail, `PlanetInHouse`.
String _pascal(String kebab) =>
    [
      for (final word in kebab.split('-'))
        if (word.isNotEmpty) word[0].toUpperCase() + word.substring(1),
    ].join();

/// The variants of a sealed request class.
List<ClassMirror> _variants(ClassMirror sealed) => [
  for (final one in _classes)
    if (one.superclass?.simpleName == sealed.simpleName) one,
];

/// The variants met, by class name.
final Set<String> _met = {};

/// [json] read as a value of [type].
Object? _read(TypeMirror type, Object? json, String at) {
  if (json == null) return null;
  final name = MirrorSystem.getName(type.simpleName);
  switch (name) {
    case 'int':
      return json as int;
    case 'double':
      return (json as num).toDouble();
    case 'num' || 'String' || 'bool' || 'Object' || 'dynamic':
      return json;
    case 'List':
      final element = type.typeArguments.first;
      final items = [
        for (final (i, item) in (json as List<Object?>).indexed)
          _read(element, item, '$at[$i]'),
      ];
      return (type as ClassMirror).newInstance(#from, [items]).reflectee;
    case 'Map':
      final element = type.typeArguments.last;
      final entries = (json as Map<String, Object?>).entries;
      return (type as ClassMirror).newInstance(#from, [
        {
          for (final MapEntry(:key, :value) in entries)
            key: _read(element, value, '$at.$key'),
        },
      ]).reflectee;
  }
  final mirror = type as ClassMirror;
  if (mirror.isEnum) {
    final key = (json as String).split('.').last;
    return (mirror.getField(#values).reflectee as List<Object?>).firstWhere(
      (member) => reflect(member).getField(#key).reflectee == key,
      orElse: () => throw StateError('$at: no $name is `$key`'),
    );
  }
  if (mirror.isAbstract) {
    final variants = _variants(mirror);
    if (json case {'type': final String tag}) {
      final variant = variants.firstWhere(
        (one) => MirrorSystem.getName(one.simpleName) == '$name${_pascal(tag)}',
        orElse: () => throw StateError('$at: no $name variant `$tag`'),
      );
      return _build(variant, json, at);
    }
    // An externally tagged variant carrying one value (`{"minor": 1}`):
    // the variant its key names, built from the value.
    if (json case Map<String, Object?>(length: 1)) {
      final MapEntry(:key, :value) = json.entries.single;
      final variant =
          variants
              .where(
                (one) =>
                    MirrorSystem.getName(one.simpleName) ==
                    '$name${_pascal(key.toLowerCase().replaceAll('_', '-'))}',
              )
              .firstOrNull;
      if (variant != null) {
        final built = _build(variant, value, '$at.$key');
        if (_agrees(_written(built), json)) return built;
      }
    }
    // An untagged union: the variant that reads the value and writes it
    // back as it was.
    for (final variant in variants) {
      try {
        final value = _build(variant, json, at);
        if (_agrees(_written(value), json)) return value;
      } on Object {
        continue;
      }
    }
    throw StateError('$at: no $name variant reads $json');
  }
  return _build(mirror, json, at);
}

/// The hand-kept classes whose parameters the wire names otherwise, each
/// parameter to its wire name. Listed rather than inferred.
const Map<String, Map<String, String>> _renamed = {
  'FestivalWindow': {'fromMuhurta': 'from', 'toMuhurta': 'to'},
};

/// The generated classes holding a record laid flat (serde's `flatten`),
/// each to the parameter that holds it: the record has no key of its own,
/// its parts being the object's. Listed rather than inferred.
const Map<String, String> _laid = {'Cancellation': 'condition'};

/// A request record of [type] from its JSON object: each named parameter
/// from its field (a Dart keyword's `Value` dropped), or one positional
/// parameter from the whole value.
Object _build(ClassMirror type, Object? json, String at) {
  final name = MirrorSystem.getName(type.simpleName);
  _met.add(name);
  final methods = type.declarations.values.whereType<MethodMirror>();
  final constructor =
      methods
          .where(
            (one) =>
                one.isConstructor && one.constructorName == const Symbol(''),
          )
          .firstOrNull;
  if (constructor == null) {
    // A value with named members and a constructor a member each (an
    // ashauch length: `full`, or `count(days)`) is read by the layer's own
    // reader, the one an answer carrying it is read with.
    final reader = methods.firstWhere(
      (one) => one.isStatic && MirrorSystem.getName(one.simpleName) == '_read',
      orElse: () => throw StateError('$at: $name has no constructor or reader'),
    );
    return type.invoke(reader.simpleName, [json]).reflectee as Object;
  }
  final positional = <Object?>[];
  final named = <Symbol, Object?>{};
  for (final parameter in constructor.parameters) {
    if (!parameter.isNamed) {
      positional.add(_read(parameter.type, json, at));
      continue;
    }
    final field = MirrorSystem.getName(parameter.simpleName);
    if (_laid[name] == field) {
      named[parameter.simpleName] = _read(parameter.type, json, at);
      continue;
    }
    final wire =
        _renamed[name]?[field] ??
        (field.endsWith('Value')
            ? field.substring(0, field.length - 'Value'.length)
            : field);
    final fields = json! as Map<String, Object?>;
    final key = fields.containsKey(field) ? field : wire;
    if (!fields.containsKey(key)) continue;
    named[parameter.simpleName] = _read(
      parameter.type,
      fields[key],
      '$at.$key',
    );
  }
  return type.newInstance(const Symbol(''), positional, named).reflectee
      as Object;
}

/// A request as the JSON it writes: its wire form, which `toJson` gives
/// for a request and a hand-written value (an ashauch event) holds alone.
Object? _written(Object request) => jsonDecode(
  jsonEncode(
    reflect(
      request,
    ).getField(MirrorSystem.getSymbol('_wire', _layer)).reflectee,
  ),
);

/// Where [written] stops saying what [source] says, or null where it
/// says it all: every field the source gives, with the same value, a
/// member written bare where the source wrote it whole. A field the source
/// leaves out may be written with the default the class gives it, as serde
/// reads it.
String? _disagreement(Object? written, Object? source, [String at = r'$']) {
  switch ((written, source)) {
    case (final Map<String, Object?> w, final Map<String, Object?> s):
      for (final MapEntry(:key, :value) in s.entries) {
        if (!w.containsKey(key)) return '$at.$key is not written';
        final found = _disagreement(w[key], value, '$at.$key');
        if (found != null) return found;
      }
      return null;
    case (final List<Object?> w, final List<Object?> s):
      if (w.length != s.length) return '$at has ${w.length}, not ${s.length}';
      for (var i = 0; i < w.length; i += 1) {
        final found = _disagreement(w[i], s[i], '$at[$i]');
        if (found != null) return found;
      }
      return null;
    case (final String w, final String s) when w == s || s.endsWith('.$w'):
      return null;
    case (final num w, final num s) when w == s:
      return null;
    case _ when written == source:
      return null;
  }
  return '$at is $written, not $source';
}

/// Where two builds of one value first differ, or null where they are
/// equal: a list member by member, a value field by field (its `_fields`,
/// which `==` compares), so a failure names the field rather than the
/// whole request.
String? _difference(Object? a, Object? b, String at) {
  if (a == b) return null;
  if ((a, b) case (final List<Object?> x, final List<Object?> y)) {
    if (x.length != y.length) return '$at has ${x.length}, not ${y.length}';
    for (var i = 0; i < x.length; i += 1) {
      final found = _difference(x[i], y[i], '$at[$i]');
      if (found != null) return found;
    }
    return '$at: lists alike member by member are unequal';
  }
  if (a != null && b != null && a.runtimeType == b.runtimeType) {
    final fields = MirrorSystem.getSymbol('_fields', _layer);
    final mirror = reflect(a);
    if (mirror.type.instanceMembers.containsKey(fields)) {
      final found = _difference(
        mirror.getField(fields).reflectee,
        reflect(b).getField(fields).reflectee,
        '$at.${a.runtimeType}',
      );
      if (found != null) return found;
    }
    return '$at: two ${a.runtimeType}s alike field by field are unequal';
  }
  return '$at is $a, not $b';
}

bool _agrees(Object? written, Object? source) =>
    _disagreement(written, source) == null;

/// [path] in the repository, found by walking up from where the suite
/// runs.
String _repo(String path) {
  for (var dir = Directory.current; ; dir = dir.parent) {
    final found = '${dir.path}/$path';
    if (FileSystemEntity.typeSync(found) != FileSystemEntityType.notFound) {
      return found;
    }
    if (dir.parent.path == dir.path) {
      throw StateError('no $path above ${Directory.current.path}');
    }
  }
}

/// The rule files.
final Directory _rules = Directory(_repo('crates/rules/rules'));

/// The records this reader cannot build, because each reaches a class the
/// layer keeps by hand in a shape of its own rather than the wire's: a
/// place as an `Observer` (`ReckonedPlace`, `ResearchBirth`), a birth date
/// by position (`BirthDate`), an orb model through its own constructors
/// (`OrbModel`). Listed, so a record that becomes buildable asks to leave.
const Set<String> _handShaped = {
  'almanac.publishedReasons',
  'chart.found',
  'numerology.profile',
  'research.study',
};

/// The class each deep example's record is written by. Listed rather than
/// inferred, so a record added to the examples asks for its class here.
const Map<String, Type> _records = {
  'almanac.ashauch': AshauchQuery,
  'almanac.changes': ChangesRequest,
  'almanac.days': AlmanacDaysRequest,
  'almanac.digest': DigestRequest,
  'almanac.festival_calendar': FestivalCalendarRequest,
  'almanac.festivalPack': PackRequest,
  'almanac.firstYear': FirstYearQuery,
  'almanac.next': NextRequest,
  'almanac.sentences': SentencesRequest,
  'almanac.pakshi': PakshiRequest,
  'almanac.publishedReasons': PublishedReasonsRequest,
  'almanac.sankalpa': SankalpaRequest,
  'calendar.add': AddAsked,
  'calendar.between': BetweenAsked,
  'calendar.convert': CalendarAsked,
  'chart.found': ChartFoundRequest,
  'chart.rashifal': BatchAsked,
  'intl.date': DateTextRequest,
  'intl.inWords': InWordsRequest,
  'intl.time': TimeTextRequest,
  'matching.naam': NaamRequest,
  'naming.check': NamingCheckRequest,
  'naming.of': NamingOfRequest,
  'naming.read': NamingReadRequest,
  'naming.suggest': NamingSuggestRequest,
  'naming.syllables': NamingSyllablesRequest,
  'numerology.profile': NumerologyRequest,
  'research.study': ResearchRequest,
  'time.civil': CivilAsked,
  'time.convert': ScaleRequest,
  'time.resolve': ResolveAsked,
};

void main() {
  test('every shipped rule is written by the request classes as its file '
      'spells it', () {
    _met.clear();
    var rules = 0;
    final files =
        _rules
            .listSync()
            .whereType<File>()
            .where((file) => file.path.endsWith('.json'))
            .toList()
          ..sort((a, b) => a.path.compareTo(b.path));
    for (final file in files) {
      final pack = jsonDecode(file.readAsStringSync());
      if (pack case {'rules': final List<Object?> shipped}) {
        for (final source in shipped) {
          final key = (source! as Map<String, Object?>)['key'];
          final rule = _read(reflectType(RuleDefinition), source, '$key')!;
          expect((rule as RuleDefinition).key, key);
          expect(
            _disagreement(_written(rule), source),
            isNull,
            reason: '$key is written as ${file.uri.pathSegments.last} has it',
          );
          final again = _read(reflectType(RuleDefinition), source, '$key');
          expect(rule, again, reason: '$key built twice is one value');
          expect(rule.hashCode, again.hashCode);
          rules += 1;
        }
      }
    }
    expect(rules, greaterThan(1000));
    final conditions = _met.where((name) => name.startsWith('Condition'));
    expect(conditions.length, greaterThan(40), reason: '$conditions');
  });

  test('every deep request example is written by the request classes as '
      'it was given', () {
    final examples =
        jsonDecode(
              File(
                _repo('crates/sdk/tests/deep-requests.json'),
              ).readAsStringSync(),
            )
            as List<Object?>;
    _met.clear();
    final failed = <String, String>{};
    for (final example in examples.cast<Map<String, Object?>>()) {
      final record = example['record']! as String;
      final source = example['request'];
      final type = _records[record];
      expect(type, isNotNull, reason: '$record has a class');
      try {
        final request = _read(reflectType(type!), source, record)!;
        expect(
          _disagreement(_written(request), source),
          isNull,
          reason: '$record is written as it was given',
        );
        final again = _read(reflectType(type), source, record);
        expect(
          _difference(request, again, record),
          isNull,
          reason: '$record built twice is one value',
        );
        expect(request.hashCode, again.hashCode);
      } on Object catch (e) {
        failed[record] = '$e'.split('\n').first;
      }
    }
    expect(
      failed.keys.toSet(),
      _handShaped,
      reason: 'every other record is built and written as given: $failed',
    );
    expect(examples, hasLength(_records.length));
    expect(_met.length, greaterThan(50), reason: 'the classes built: $_met');
  });

  test('equal values hash alike, however deep their lists', () {
    // `==` compares nested lists member by member, so the hash must too:
    // a value with a list of lists, built twice, is one key of a set.
    VarshphalRows rows() => VarshphalRows(
      rows: [
        [1, 2],
        [3],
      ],
    );
    expect(rows(), rows());
    expect(rows().hashCode, rows().hashCode);
    expect({rows()}.contains(rows()), isTrue);
    expect(
      rows(),
      isNot(
        const VarshphalRows(
          rows: [
            [1, 2],
            [4],
          ],
        ),
      ),
    );
  });

  test('a condition differing in one field is another condition', () {
    const one = ConditionPlanetInHouse(houses: [9], planet: 'JUPITER');
    expect(one, const ConditionPlanetInHouse(houses: [9], planet: 'JUPITER'));
    expect(
      one,
      isNot(const ConditionPlanetInHouse(houses: [10], planet: 'JUPITER')),
    );
    expect(
      const ConditionNot(condition: one),
      isNot(
        const ConditionNot(
          condition: ConditionPlanetCombust(planet: 'JUPITER'),
        ),
      ),
    );
    expect(jsonDecode(jsonEncode(const ConditionNot(condition: one))), {
      'type': 'not',
      'condition': {
        'type': 'planet-in-house',
        'houses': [9],
        'planet': 'JUPITER',
      },
    });
  });

  test('a cancellation writes its condition with its label beside it', () {
    const burnt = Cancellation(
      condition: ConditionPlanetCombust(planet: 'MARS'),
      label: 'Mars burnt',
    );
    expect(jsonDecode(jsonEncode(burnt)), {
      'type': 'planet-combust',
      'planet': 'MARS',
      'label': 'Mars burnt',
    });
    expect(
      jsonDecode(
        jsonEncode(const Cancellation(condition: ConditionBirthByDay())),
      ),
      {'type': 'birth-by-day'},
    );
    expect(
      burnt,
      isNot(
        const Cancellation(
          condition: ConditionPlanetCombust(planet: 'VENUS'),
          label: 'Mars burnt',
        ),
      ),
    );
  });

  test('a request the layer keeps by hand is a value: equal when it writes '
      'the same JSON', () {
    final here = Observer(
      longitudeDeg: Longitude(85.3),
      latitudeDeg: Latitude(27.7),
      altitudeM: Altitude(1400),
    );
    final there = Observer(
      longitudeDeg: Longitude(77.2),
      latitudeDeg: Latitude(28.6),
      altitudeM: Altitude(216),
    );
    // Each built twice, not as a constant, so the two are not one object;
    // and another of the class that writes something else.
    final cases = <(Object Function(), Object)>[
      (() => AnnualPlace.at(here, utcOffsetSeconds: 20700), AnnualPlace.birth),
      (() => AntisciaRequest(), const AntisciaRequest(orbs: OrbModel.leo)),
      (
        () => AshtakavargaAyusRules(divisor: AyusDivisor.thirty),
        const AshtakavargaAyusRules(divisor: AyusDivisor.sevenOverTwentySeven),
      ),
      (() => AttendantFacts(total: 3), const AttendantFacts(total: 4)),
      (
        () => AyurdayaRules(combine: AyurdayaCombine.largest),
        const AyurdayaRules(combine: AyurdayaCombine.each),
      ),
      (
        () => BaselineDashaRules(seedOverflow: SeedOverflow.reject),
        const BaselineDashaRules(seedOverflow: SeedOverflow.wrapToStart),
      ),
      (
        () => BaselineRectificationRequest(uncertaintyMinutes: 30),
        const BaselineRectificationRequest(uncertaintyMinutes: 60),
      ),
      (
        () => BirthFacts(fatherPresent: true),
        const BirthFacts(fatherPresent: false),
      ),
      (
        () => CircumstanceRequest(facts: BirthFacts(fatherPresent: true)),
        const CircumstanceRequest(),
      ),
      (
        () => DignityRequest(sectRule: SectRule.horizon),
        const DignityRequest(sectRule: SectRule.daylight),
      ),
      (
        () => DossierRequest(dashaDepth: 2),
        const DossierRequest(dashaDepth: 3),
      ),
      (
        () => FestivalRequest(rules: FestivalPack.nepal),
        const FestivalRequest(rules: FestivalPack.dharmasindhu),
      ),
      (() => HarmonicRequest(5), const HarmonicRequest(7)),
      (
        () => HitRequest(from: 2460000.5, to: 2460030.5),
        const HitRequest(from: 2460000.5, to: 2460060.5),
      ),
      (() => Interval(from: 1.0, to: 2.0), const Interval(from: 1.0, to: 3.0)),
      (() => LalKitabRequest(year: 30), const LalKitabRequest(year: 31)),
      (
        () => LifeEvent(kind: LifeEventKind.marriage, on: 2460000.5),
        const LifeEvent(kind: LifeEventKind.engagement, on: 2460000.5),
      ),
      (
        () => MasaPraveshaRequest(year: 2026),
        const MasaPraveshaRequest(year: 2027),
      ),
      (
        () => MatchingRequest(
          Partner(instant: 2460000.5, place: here),
          partnerRole: MatchRole.bride,
        ),
        MatchingRequest(
          Partner(instant: 2460000.5, place: here),
          partnerRole: MatchRole.groom,
        ),
      ),
      (
        () => MuhurtaRequest(rules: MuhurtaActivity.ramanMarriage),
        const MuhurtaRequest(
          rules: MuhurtaActivity.ramanMarriage,
          daysWithWindows: 9,
        ),
      ),
      (() => NatalGraha(Graha.mars), const NatalLagna()),
      (
        () => Partner(instant: 2460000.5, place: here),
        Partner(instant: 2460000.5, place: there),
      ),
      (() => PrashnaRequest(house: 7), const PrashnaRequest(house: 10)),
      (
        () => ProgressionContacts(from: 2460000.5, to: 2460365.5),
        const ProgressionContacts(from: 2460000.5, to: 2460365.5, mutual: true),
      ),
      (() => ProgressionsRequest(at: 2460676.5), const ProgressionsRequest()),
      (
        () => PurifierRules(seedMinutes: 5.0),
        const PurifierRules(seedMinutes: 10.0),
      ),
      (() => PurifyRequest(minutes: 30), const PurifyRequest(minutes: 60)),
      (
        () => RasmiRules(place: RayPlace.sign),
        const RasmiRules(place: RayPlace.dwadasamsa),
      ),
      (
        () => RectificationRequest(purify: PurifyRequest(minutes: 30)),
        const RectificationRequest(),
      ),
      (() => RuleRequest(longevity: true), const RuleRequest()),
      (
        () => ThreePairsRules(basis: PairsBasis.perPair),
        const ThreePairsRules(basis: PairsBasis.classYears),
      ),
      (() => VarshaRequest(through: 30), const VarshaRequest(through: 31)),
      (
        () => VarsheshaRules(subDegree: SubDegree.poorna),
        const VarsheshaRules(subDegree: SubDegree.none),
      ),
      (
        () => WesternAspectRequest(aspects: [WesternAspect.conjunction]),
        const WesternAspectRequest(aspects: [WesternAspect.semiSextile]),
      ),
      (() => YearClock.days(360), const YearClock.days(365)),
      (
        () => YogaRules(subDegree: SubDegree.poorna),
        const YogaRules(subDegree: SubDegree.none),
      ),
    ];
    for (final (build, other) in cases) {
      final (a, b) = (build(), build());
      final name = a.runtimeType;
      expect(identical(a, b), isFalse, reason: '$name built twice');
      expect(a, b, reason: '$name');
      expect(a.hashCode, b.hashCode, reason: '$name');
      expect({a}.contains(b), isTrue, reason: '$name');
      expect(a, isNot(other), reason: '$name');
    }
    expect(cases, hasLength(36));
  });
}
