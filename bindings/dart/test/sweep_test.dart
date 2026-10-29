// The tests that walk a whole generated surface rather than one call:
// every typed message accessor against the locale engine, and every
// catalogue member against the library's key table. A generator that
// writes one accessor or one member wrong fails here, whichever it is.
//
// The surfaces are walked by reflection, so a member added tomorrow is
// held the day it is generated.

import 'dart:mirrors';

import 'package:teistro/messages.dart' as intl;
import 'package:teistro/teistro.dart';
import 'package:test/test.dart';

final Teistro teistro = Teistro.open();

Context context() => teistro.context(
  profile: 'nepali-default',
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
Object _argument(ParameterMirror parameter) {
  final name = MirrorSystem.getName(parameter.simpleName);
  final type = parameter.type;
  if (type is ClassMirror && type.isEnum) {
    return (type.getField(#values).reflectee as List<Object?>).first!;
  }
  return switch (MirrorSystem.getName(type.simpleName)) {
    'int' => 1,
    'num' || 'double' => 2460000.25,
    'String' => name == 'zone' ? 'Asia/Kathmandu' : 'x',
    'List' => const <Object>['x'],
    final other => throw StateError('no argument for $name, a $other'),
  };
}

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

/// Walks the accessor tree under [node], holding each message accessor
/// to a key the locale renders whole, with every parameter it sends, and
/// each entity accessor to forms with a name.
void _walk(Object node, _Checked checked, _Swept swept) {
  final mirror = reflect(node);
  for (final declaration
      in mirror.type.declarations.values.whereType<MethodMirror>()) {
    if (declaration.isStatic ||
        declaration.isConstructor ||
        declaration.isPrivate ||
        declaration.isSetter) {
      continue;
    }
    final name =
        '${MirrorSystem.getName(mirror.type.simpleName)}.'
        '${MirrorSystem.getName(declaration.simpleName)}';
    final returns = MirrorSystem.getName(declaration.returnType.simpleName);
    if (declaration.isRegularMethod && returns == 'String') {
      final named = {
        for (final parameter in declaration.parameters)
          parameter.simpleName: _argument(parameter),
      };
      final text =
          mirror.invoke(declaration.simpleName, const [], named).reflectee
              as String;
      final key = checked.key!;
      expect(swept.messages.add(key), isTrue, reason: '$key reached twice');
      expect(checked.ctx.intl.has(key), isTrue, reason: '$name names $key');
      expect(checked.params!.keys.toSet(), {
        // A Dart keyword (`class`) is a parameter with a `$` after it.
        for (final p in declaration.parameters)
          MirrorSystem.getName(p.simpleName).replaceAll(r'$', ''),
      }, reason: '$name sends every parameter by its name');
      final unexpected = [
        for (final warning in checked.warnings)
          if (!_expected(key, warning)) warning,
        if (checked.fallback) 'rendered from a fallback locale',
      ];
      if (unexpected.isNotEmpty) swept.unexpected[key] = unexpected;
      expect(text, isNot(anyOf(isEmpty, key)), reason: '$name says something');
    } else if (declaration.isGetter && returns == 'EntityForms') {
      final forms =
          mirror.getField(declaration.simpleName).reflectee as intl.EntityForms;
      final key = checked.key!;
      expect(swept.entities.add(key), isTrue, reason: '$key reached twice');
      expect(forms.name, isNotEmpty, reason: '$key has a name');
      expect(forms.forms['name'], forms.name, reason: 'every form is kept');
    } else if (declaration.isGetter && returns.startsWith('Messages')) {
      _walk(
        mirror.getField(declaration.simpleName).reflectee as Object,
        checked,
        swept,
      );
    }
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
    final library = reflectClass(Kind).owner! as LibraryMirror;
    var enums = 0;
    var members = 0;
    for (final type in library.declarations.values
        .whereType<ClassMirror>()
        .where(
          (type) => type.isEnum && type.staticMembers.containsKey(#byId),
        )) {
      enums += 1;
      final name = MirrorSystem.getName(type.simpleName);
      final open = type.staticMembers.containsKey(#registered);
      Object? byId(int id) => type.invoke(#byId, [id]).reflectee;
      Object? byKey(String key) => type.invoke(#byKey, [key]).reflectee;
      final ids = <int>{};
      String? kind;
      for (final member in type.getField(#values).reflectee as List<Object?>) {
        final fields = reflect(member);
        final id = fields.getField(#id).reflectee as int;
        final key = fields.getField(#key).reflectee as String;
        if (open && key == 'UNKNOWN') {
          expect(id, -1, reason: '$name.unknown');
          continue;
        }
        members += 1;
        expect(ids.add(id), isTrue, reason: '$name.$key has its own id');
        expect(byId(id), same(member), reason: '$name.$key by its id');
        expect(byKey(key), same(member), reason: '$name.$key by its key');
        if (member is KeyOf<Object?>) {
          final full = member.fullKey;
          kind = full.substring(0, full.indexOf('.'));
          expect(full, '$kind.$key', reason: "$name.$key's full key");
          expect(byKey(full), same(member), reason: '$full by its full key');
          final family = Kind.byKey(kind);
          expect(family, isNotNull, reason: '$kind is a kind');
          final packed = (family!.id << 16) | id;
          expect(ctx.keys.id(full), packed, reason: "$full is the library's");
          expect(ctx.keys.name(packed), full);
        }
      }
      expect(byKey('NOT_A_MEMBER_OF_ANY_KIND'), isNull, reason: name);
      const stranger = 0x7FFE;
      if (open) {
        final unknown = byId(stranger)!;
        expect(reflect(unknown).getField(#key).reflectee, 'UNKNOWN');
        final made =
            type.invoke(#registered, ['ACME']).reflectee as KeyOf<Object?>;
        expect(made.fullKey, '$kind.ACME', reason: '$name registers its kind');
        expect(made.toString(), made.fullKey);
        final again = type.invoke(#registered, ['ACME']).reflectee;
        expect(made, again, reason: 'a registered member is its key');
        expect(made.hashCode, again.hashCode);
        expect(made, isNot(type.invoke(#registered, ['OTHER']).reflectee));
      } else {
        expect(
          () => byId(stranger),
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
  });
}
