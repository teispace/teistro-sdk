// The typed accessors: every message of the SDK's own locale as a
// function of its parameters, and every catalogued entity as its forms.

import 'package:teistro/messages.dart' as intl;
import 'package:teistro/teistro.dart';
import 'package:test/test.dart';

final Teistro teistro = Teistro.open();

Context context() => teistro.context(
  profile: 'nepali-default',
  locale: 'ne-Deva-NP',
  testProvider: true,
);

void main() {
  test('a message is spelled by its accessor, never by its key', () {
    final ctx = context();
    addTearDown(ctx.dispose);
    expect(
      ctx.messages.sdk.reason.grahaInBhava(
        graha: intl.GrahaKey.jupiter,
        bhava: 7,
      ),
      ctx.render('sdk.reason.grahaInBhava', {
        'graha': {r'$entity': 'graha.JUPITER'},
        'bhava': 7,
      }).text,
      reason: 'the accessor wraps the entity as the engine takes it',
    );
    expect(
      ctx.messages.sdk.reason.grahaInBhava(
        graha: intl.GrahaKey.jupiter,
        bhava: 7,
      ),
      contains('७'),
    );

    ctx.locale = 'en-Latn';
    expect(
      ctx.messages.sdk.calendar.bikramSambat.date.long(
        day: 1,
        monthName: 'Baisakh',
        year: 2072,
      ),
      '1 Baisakh 2072 BS',
    );
    expect(
      ctx.messages.sdk.calendar.gregorian.date.numeric(
        day: 14,
        month: 4,
        year: 2015,
      ),
      '2015-04-14',
    );
  });

  test("an entity's forms come from the locale, not from the caller", () {
    final ctx = context();
    addTearDown(ctx.dispose);
    final sun = ctx.entity('graha.SUN');
    expect(sun.name, 'सूर्य');
    expect(sun.iast, 'Sūrya');
    expect(sun.glyph, '☉');
    expect(sun.gender, intl.Gender.m);
    expect(
      ctx.messages.sdk.entity.graha.sun.name,
      sun.name,
      reason: 'the accessor reads the same forms',
    );

    ctx.locale = 'en-Latn';
    expect(ctx.entity('graha.SUN').name, 'Sun');
    expect(ctx.entity('rashi.ARIES').name, 'Aries');
  });

  test('an entity the locale does not carry is refused by name', () {
    final ctx = context();
    addTearDown(ctx.dispose);
    expect(
      () => ctx.entity('graha.PLUTO'),
      throwsA(
        isA<TeistroException>()
            .having((e) => e.status, 'status', Status.unsupported)
            .having((e) => e.field, 'field', 'key'),
      ),
    );
  });

  test('a term written in one script reads in the other', () {
    final ctx = context();
    addTearDown(ctx.dispose);
    expect(ctx.transliterate('सूर्य बृहस्पति'), 'sūrya bṛhaspati');
    expect(ctx.transliterate(ctx.entity('graha.MARS').name), 'maṅgala');
    expect(
      ctx.transliterate('Jupiter'),
      'Jupiter',
      reason: 'what is not the script passes through',
    );
    expect(
      () => ctx.transliterate('x', from: 'iast', to: 'deva'),
      throwsA(
        isA<TeistroException>().having(
          (e) => e.status,
          'status',
          Status.unsupported,
        ),
      ),
    );
    expect(
      () => ctx.transliterate('x', to: 'taml'),
      throwsA(isA<TeistroException>().having((e) => e.field, 'field', 'to')),
    );
  });

  test('the accessors are a tree over any renderer', () {
    final asked = <(String, Map<String, Object?>)>[];
    final tree = intl.Messages(_Recording(asked));
    expect(tree.sdk.reason.welcome(), 'sdk.reason.welcome');
    expect(tree.sdk.entity.graha.ketu.name, 'graha.KETU');
    tree.sdk.calendar.ghati.long(ghati: 12, pala: 30);
    expect(asked.last.$1, 'sdk.calendar.ghati.long');
    expect(asked.last.$2, {'ghati': 12, 'pala': 30});
    expect(
      intl.EntityForms.of('{"name":"x"}').iast,
      isEmpty,
      reason: 'a form the locale lacks is empty',
    );
  });

  test('a date, a time and a ghati count reach the engine tagged', () {
    final asked = <(String, Map<String, Object?>)>[];
    final tree = intl.Messages(_Recording(asked));
    tree.sdk.calendar.datetime.join(date: 'a', time: 'b');
    expect(asked.last.$2, {'date': 'a', 'time': 'b'});
    tree.sdk.reason.grahaAt(graha: intl.GrahaKey.sun, longitude: 12.5);
    expect(asked.last.$2['graha'], {r'$entity': 'graha.SUN'});
    expect(asked.last.$2['longitude'], 12.5);
  });
}

/// A renderer that records what it was asked for.
final class _Recording implements intl.Renderer {
  _Recording(this.asked);

  final List<(String, Map<String, Object?>)> asked;

  @override
  String render(String key, [Map<String, Object?> params = const {}]) {
    asked.add((key, params));
    return key;
  }

  @override
  intl.EntityForms entity(String key) => intl.EntityForms.of({'name': key});
}
