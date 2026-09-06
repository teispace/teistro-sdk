/// The Teistro SDK's typed message accessors: every message of the SDK's
/// own locale as a function of its parameters, and every catalogued
/// entity as its forms.
///
/// Its own entry point, because the names here are the locale's rather
/// than the catalogue's and one of them (`Gender`) is a word both use:
///
/// ```dart
/// import 'package:teistro/teistro.dart';
/// import 'package:teistro/messages.dart' as intl;
///
/// final text = ctx.messages.sdk.reason.grahaInBhava(
///   graha: intl.GrahaKey.jupiter,
///   bhava: 7,
/// );
/// ```
library;

export 'src/messages.dart';
