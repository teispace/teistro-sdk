/// The shapes the generated surface tables describe the library in, so the
/// tests that walk a whole surface read it from a table rather than by
/// reflection: `library.g.dart` (`cargo xtask gen ffi`) lists the
/// catalogue's enums and the request types, and `messages.g.dart`
/// (`cargo xtask gen intl`) the typed message accessors. Each table is
/// rendered from the description its part of the library is rendered from,
/// and the generators' gates hold each to its generator, so a member added
/// tomorrow is walked the day it is generated.
///
/// Dart's reflection (`dart:mirrors`) is deprecated from Dart 3.14, which is
/// why the tables exist. A constructor or an accessor is reached here as a
/// tear-off and called through [Function.apply], which is not reflection.
library;

import 'package:teistro/messages.dart' as intl;

/// A type a table names.
abstract class Surface {
  const Surface(this.name);

  /// The type's name, as the library declares it.
  final String name;
}

/// An enum read by its members' keys.
class KeySet<T extends Enum> extends Surface {
  const KeySet(super.name, this.values, this._key);

  /// Its members, in order.
  final List<T> values;

  final String Function(T member) _key;

  /// The key [member] is spelt with.
  String keyOf(Object? member) => _key(member as T);
}

/// An enum of `catalogue.dart`, with its static readers.
final class Catalogued<T extends Enum> extends KeySet<T> {
  const Catalogued(
    String name,
    List<T> values,
    this._byId,
    this._byKey,
    this._id,
    String Function(T member) key, [
    this._registered,
  ]) : super(name, values, key);

  final T Function(int id) _byId;
  final T? Function(String key) _byKey;
  final int Function(T member) _id;
  final Object Function(String key)? _registered;

  /// Its `byId`.
  T byId(int id) => _byId(id);

  /// Its `byKey`.
  T? byKey(String key) => _byKey(key);

  /// The id [member] carries.
  int idOf(Object? member) => _id(member as T);

  /// Whether a context may register a member of its kind: a catalogued
  /// kind's enum has `registered`, a closed set's has not.
  bool get open => _registered != null;

  /// Its `registered`.
  Object registered(String key) => _registered!(key);
}

/// A union's root: what a value of it is, is one of its variants.
final class Sealed extends Surface {
  const Sealed(super.name, {this.pick});

  /// For a union the layer keeps by hand, which no test can write back to
  /// see whether a variant agrees: the variant [json] is, by name, or null
  /// for none. A generated union has none, and its variants are tried.
  final String? Function(Object? json)? pick;
}

/// A class built by its constructor.
final class Built<T extends Object> extends Surface {
  const Built(
    super.name,
    this.make,
    this.params, {
    this.parent,
    List<Object?> Function(T value)? fields,
  }) : _fields = fields;

  /// The constructor, torn off.
  final Function make;

  /// Its parameters, in order.
  final List<Param> params;

  /// The union it is a variant of, by name.
  final String? parent;

  final List<Object?> Function(T value)? _fields;

  /// The fields [value]'s equality compares, in order; null for a class
  /// the table does not know them for.
  List<Object?>? fieldsOf(Object value) => _fields?.call(value as T);

  /// The class built from [positional] and [named], a named parameter
  /// left out taking its default.
  Object build(List<Object?> positional, Map<String, Object?> named) =>
      Function.apply(make, positional, {
            for (final MapEntry(:key, :value) in named.entries)
              Symbol(key): value,
          })
          as Object;
}

/// A value the layer reads with a reader of its own rather than a
/// constructor.
final class Read extends Surface {
  const Read(super.name, this.read);

  /// The value [json] is.
  final Object Function(Object? json) read;
}

/// A parameter: its name as Dart spells it, and how its value is read.
final class Param {
  const Param(this.name, this.shape, {this.named = true});

  final String name;
  final Shape shape;

  /// Whether it is named rather than positional.
  final bool named;
}

/// How a parameter's value is read.
sealed class Shape {
  const Shape();
}

/// A value of a core type, by its name (`int`, `double`, `String`, `bool`,
/// `Object`) or a value class's (`TimeValue`).
final class Scalar extends Shape {
  const Scalar(this.name);

  final String name;
}

/// A value of a type a table names.
final class Named extends Shape {
  const Named(this.name);

  final String name;
}

/// A list, and how it is made a list of its items' type.
final class ListOf extends Shape {
  const ListOf(this.item, this.typed);

  final Shape item;
  final List<Object?> Function(List<Object?> items) typed;
}

/// A map from text, and how it is made a map of its values' type.
final class MapOf extends Shape {
  const MapOf(this.value, this.typed);

  final Shape value;
  final Map<String, Object?> Function(Map<String, Object?> entries) typed;
}

/// A member of an enum, given by its members.
final class Members extends Shape {
  const Members(this.values);

  final List<Enum> values;
}

/// A type no test reads from JSON (a registered key, a pair), by its Dart
/// spelling.
final class Unread extends Shape {
  const Unread(this.dart);

  final String dart;
}

/// A message accessor: the method a walk from the root reaches, torn off.
final class MessageSurface {
  const MessageSurface(this.name, this.accessor, this.params);

  /// Its class and name (`MessagesSdkAspect.cast`).
  final String name;

  /// The method, torn off the accessors at [intl.Messages].
  final Function Function(intl.Messages messages) accessor;

  /// Its named parameters, in order.
  final List<Param> params;
}

/// An entity accessor: the getter a walk from the root reaches.
final class EntitySurface {
  const EntitySurface(this.name, this.accessor);

  /// Its class and name.
  final String name;

  /// The getter, read from the accessors at [intl.Messages].
  final intl.EntityForms Function(intl.Messages messages) accessor;
}
