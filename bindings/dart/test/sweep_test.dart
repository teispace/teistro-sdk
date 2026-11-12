// The tests that walk a whole generated surface rather than one call:
// every typed message accessor against the locale engine, and every
// catalogue member against the library's key table. A generator that
// writes one accessor or one member wrong fails here, whichever it is.
//
// The surfaces are walked from tables generated beside them
// (`surface/messages.g.dart`, `surface/library.g.dart`), so a member added
// tomorrow is held the day it is generated.

import 'package:teistro/messages.dart' as intl;
import 'package:teistro/teistro.dart';
import 'package:test/test.dart';

import 'surface/library.g.dart' as surface;
import 'surface/messages.g.dart' as accessors;
import 'surface/shapes.dart';

final Teistro teistro = Teistro.open();

Context context() => teistro.context(
  profile: 'nepali-drik',
  locale: 'ne-Deva-NP',
  testProvider: true,
);

/// A renderer that answers through the context and keeps what it was
/// asked and what the engine warned.
final class _Checked implements intl.Renderer {
  _Checked(this.ctx);

  final Context ctx;
  String? key;
  Map<String, Object?>? params;
  List<String> warnings = const [];
  bool fallback = false;

  @override
  String render(String key, [Map<String, Object?> params = const {}]) {
    this.key = key;
    this.params = params;
    final rendered = ctx.intl.render(key, params.isEmpty ? null : params);
    warnings = rendered.warningList;
    fallback = rendered.isFallback != 0;
    return rendered.text;
  }

  @override
  intl.EntityForms entity(String key) {
    this.key = key;
    params = null;
    return ctx.intl.entity(key);
  }
}

/// A value of an accessor's parameter: a member of an enum, the zone a
/// zone names, an instant for a number, and text otherwise.
Object _argument(Param parameter) => switch (parameter.shape) {
  Members(:final values) => values.first,
  Scalar(name: 'int') => 1,
  Scalar(name: 'num' || 'double') => 2460000.25,
  Scalar(name: 'String') => parameter.name == 'zone' ? 'Asia/Kathmandu' : 'x',
  Scalar(name: 'List') => const <Object>['x'],
  Scalar(name: 'TimeValue') => const intl.TimeValue(hour: 6, minute: 5),
  final other =>
    throw StateError('no argument for ${parameter.name}, a $other'),
};

/// Whether the engine warning [warning], rendering [key] in any locale
/// with no pack loaded, is one the walk expects. Text parameters get the
/// stand-in `x`, which a message reading the parameter as an entity a
/// pack brings (a rule, a house's reading) refuses by name. A phala
/// message reads a form of its entity (`phala`, `dashaActivation`) that
/// only an interpretation pack brings, and the `state` kind has no entry
/// in either shipped locale at all.
bool _expected(String key, String warning) =>
    warning.contains('`x`') ||
    (key.startsWith('sdk.phala.') &&
        (RegExp(
              r'^entity `[a-z_]+\.[A-Z0-9_]+` has no `\w+` form$',
            ).hasMatch(warning) ||
            warning.startsWith('missing entity `state.')));

/// What a walk of the accessors met.
final class _Swept {
  final messages = <String>{};
  final entities = <String>{};

  /// Each warning the walk did not expect, by key.
  final unexpected = <String, List<String>>{};
}

/// Walks every accessor from [root], holding each message accessor to a
/// key the locale renders whole, with every parameter it sends, and each
/// entity accessor to forms with a name.
void _walk(intl.Messages root, _Checked checked, _Swept swept) {
  for (final message in accessors.messages) {
    final name = message.name;
    final text =
        Function.apply(message.accessor(root), const [], {
              for (final parameter in message.params)
                Symbol(parameter.name): _argument(parameter),
            })
            as String;
    final key = checked.key!;
    expect(swept.messages.add(key), isTrue, reason: '$key reached twice');
    expect(checked.ctx.intl.has(key), isTrue, reason: '$name names $key');
    expect(checked.params!.keys.toSet(), {
      // A Dart keyword (`class`) is a parameter with a `$` after it.
      for (final p in message.params) p.name.replaceAll(r'$', ''),
    }, reason: '$name sends every parameter by its name');
    final unexpected = [
      for (final warning in checked.warnings)
        if (!_expected(key, warning)) warning,
      if (checked.fallback) 'rendered from a fallback locale',
    ];
    if (unexpected.isNotEmpty) swept.unexpected[key] = unexpected;
    expect(text, isNot(anyOf(isEmpty, key)), reason: '$name says something');
  }
  for (final entity in accessors.entities) {
    final forms = entity.accessor(root);
    final key = checked.key!;
    expect(swept.entities.add(key), isTrue, reason: '$key reached twice');
    expect(forms.name, isNotEmpty, reason: '$key has a name');
    expect(forms.forms['name'], forms.name, reason: 'every form is kept');
  }
}

void main() {
  for (final locale in ['en-Latn', 'ne-Deva-NP']) {
    test('every message accessor reaches a key $locale renders whole', () {
      final ctx = context();
      addTearDown(ctx.dispose);
      ctx.intl.locale = locale;
      final swept = _Swept();
      final checked = _Checked(ctx);
      _walk(intl.Messages(checked), checked, swept);
      expect(
        swept.unexpected,
        isEmpty,
        reason: 'every key takes what its accessor sends',
      );
      expect(swept.messages.length, greaterThan(140));
      expect(swept.entities.length, greaterThan(500));
    });
  }

  test(
    "an entity's forms read from text or an object, a gender when known",
    () {
      final forms = intl.EntityForms.of(
        '{"name":"Sun","iast":"Sūrya","short":"Su","glyph":"☉",'
        '"gender":"m","extra":"x","count":3}',
      );
      expect(forms.name, 'Sun');
      expect(forms.short, 'Su');
      expect(forms.gender, intl.Gender.m);
      expect(forms.forms['extra'], 'x', reason: 'an open form is kept');
      expect(forms.forms.containsKey('count'), isFalse);
      final bare = intl.EntityForms.of(<String, Object?>{'gender': 'q'});
      expect(bare.prose, isEmpty);
      expect(bare.gender, isNull, reason: 'a gender the build lacks is none');
      expect(bare.glyph, isNull);
    },
  );

  test(
    'a date, a time and a ghati count are written as the engine reads them',
    () {
      const date = intl.DateValue(
        calendar: 'gregorian',
        year: 2015,
        month: 4,
        day: 14,
      );
      const time = intl.TimeValue(hour: 5, minute: 45);
      expect(date.json, {
        'calendar': 'gregorian',
        'year': 2015,
        'month': 4,
        'day': 14,
      });
      expect(time.json, {'hour': 5, 'minute': 45, 'second': 0});
      expect(const intl.DateTimeValue(date: date, time: time).json, {
        'date': date.json,
        'time': time.json,
      });
      expect(const intl.GhatiValue(ghati: 12, pala: 30).json, {
        'ghati': 12,
        'pala': 30,
        'vipala': 0,
      });
    },
  );

  test('every catalogue member reads back by its id and key, and is the '
      "library's packed key", () {
    final ctx = context();
    addTearDown(ctx.dispose);
    var enums = 0;
    var members = 0;
    final kinds = <Kind>{};
    for (final type in surface.catalogue) {
      enums += 1;
      final name = type.name;
      final open = type.open;
      final ids = <int>{};
      String? kind;
      for (final member in type.values) {
        final id = type.idOf(member);
        final key = type.keyOf(member);
        if (open && key == 'UNKNOWN') {
          expect(id, -1, reason: '$name.unknown');
          continue;
        }
        members += 1;
        expect(ids.add(id), isTrue, reason: '$name.$key has its own id');
        expect(type.byId(id), same(member), reason: '$name.$key by its id');
        expect(type.byKey(key), same(member), reason: '$name.$key by its key');
        if (member case final KeyOf<Object?> keyed) {
          final full = keyed.fullKey;
          final family = full.substring(0, full.indexOf('.'));
          kind = family;
          expect(full, '$family.$key', reason: "$name.$key's full key");
          expect(
            type.byKey(full),
            same(member),
            reason: '$full by its full key',
          );
          final known = Kind.byKey(family);
          expect(known, isNotNull, reason: '$family is a kind');
          kinds.add(known!);
          final packed = (known.id << 16) | id;
          expect(ctx.keys.id(full), packed, reason: "$full is the library's");
          expect(ctx.keys.name(packed), full);
        }
      }
      expect(type.byKey('NOT_A_MEMBER_OF_ANY_KIND'), isNull, reason: name);
      const stranger = 0x7FFE;
      if (open) {
        final unknown = type.byId(stranger);
        expect(type.keyOf(unknown), 'UNKNOWN');
        final made = type.registered('ACME') as KeyOf<Object?>;
        expect(made.fullKey, '$kind.ACME', reason: '$name registers its kind');
        expect(made.toString(), made.fullKey);
        final again = type.registered('ACME');
        expect(made, again, reason: 'a registered member is its key');
        expect(made.hashCode, again.hashCode);
        expect(made, isNot(type.registered('OTHER')));
      } else {
        expect(
          () => type.byId(stranger),
          throwsA(
            isA<ArgumentError>().having(
              (e) => e.message,
              'message',
              contains(name),
            ),
          ),
        );
      }
    }
    expect(enums, greaterThan(140));
    expect(members, greaterThan(1000));
    // Every kind's enum is walked: one the table left out leaves its kind
    // unmet here, where a count would not notice.
    expect(kinds, Kind.values.toSet(), reason: 'every kind has its enum');
  });
}
