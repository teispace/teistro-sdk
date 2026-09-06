/// SHA-256, because the installer has to check a download and the package
/// has no dependencies to check it with.
///
/// The Dart SDK ships no digest, and the one package that would supply it
/// would be a dependency every consumer of this SDK inherits for the sake
/// of a command most of them run once. It is eighty lines of the published
/// algorithm (FIPS 180-4), tested against the standard vectors and against
/// the digest the release recorded for the library itself.
library;

import 'dart:typed_data';

/// The round constants: the first thirty-two bits of the fractional parts
/// of the cube roots of the first sixty-four primes.
const List<int> _k = <int>[
  0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, //
  0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
  0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
  0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
  0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
  0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
  0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
  0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
  0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
  0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
  0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
  0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
  0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
  0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
  0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
  0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// The initial state: the first thirty-two bits of the fractional parts of
/// the square roots of the first eight primes.
const List<int> _initial = <int>[
  0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, //
  0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

const int _mask = 0xffffffff;

int _rotr(int value, int bits) =>
    ((value >> bits) | (value << (32 - bits))) & _mask;

/// The SHA-256 of [bytes], as sixty-four lowercase hexadecimal digits.
///
/// ```dart
/// sha256Hex(const <int>[]) ==
///     'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855';
/// ```
String sha256Hex(List<int> bytes) {
  final digest = sha256(bytes);
  final out = StringBuffer();
  for (final byte in digest) {
    out.write(byte.toRadixString(16).padLeft(2, '0'));
  }
  return out.toString();
}

/// The SHA-256 of [bytes], as thirty-two bytes.
Uint8List sha256(List<int> bytes) {
  final length = bytes.length;
  // The message, a single 1 bit, zeroes to 56 modulo 64, and the length in
  // bits as a big-endian 64-bit number.
  final padded = Uint8List(((length + 9 + 63) ~/ 64) * 64)
    ..setRange(0, length, bytes);
  padded[length] = 0x80;
  final bits = length * 8;
  for (var i = 0; i < 8; i++) {
    padded[padded.length - 1 - i] = (bits >> (8 * i)) & 0xff;
  }

  final state = List<int>.of(_initial);
  final schedule = Int32List(64);
  final view = ByteData.sublistView(padded);
  for (var block = 0; block < padded.length; block += 64) {
    for (var i = 0; i < 16; i++) {
      schedule[i] = view.getUint32(block + i * 4);
    }
    for (var i = 16; i < 64; i++) {
      final a = schedule[i - 15] & _mask;
      final b = schedule[i - 2] & _mask;
      final s0 = _rotr(a, 7) ^ _rotr(a, 18) ^ (a >> 3);
      final s1 = _rotr(b, 17) ^ _rotr(b, 19) ^ (b >> 10);
      schedule[i] = (schedule[i - 16] + s0 + schedule[i - 7] + s1) & _mask;
    }

    var a = state[0];
    var b = state[1];
    var c = state[2];
    var d = state[3];
    var e = state[4];
    var f = state[5];
    var g = state[6];
    var h = state[7];
    for (var i = 0; i < 64; i++) {
      final s1 = _rotr(e, 6) ^ _rotr(e, 11) ^ _rotr(e, 25);
      final choice = (e & f) ^ (~e & g);
      final temp1 = (h + s1 + choice + _k[i] + (schedule[i] & _mask)) & _mask;
      final s0 = _rotr(a, 2) ^ _rotr(a, 13) ^ _rotr(a, 22);
      final majority = (a & b) ^ (a & c) ^ (b & c);
      final temp2 = (s0 + majority) & _mask;
      h = g;
      g = f;
      f = e;
      e = (d + temp1) & _mask;
      d = c;
      c = b;
      b = a;
      a = (temp1 + temp2) & _mask;
    }
    final round = <int>[a, b, c, d, e, f, g, h];
    for (var i = 0; i < 8; i++) {
      state[i] = (state[i] + round[i]) & _mask;
    }
  }

  final digest = Uint8List(32);
  final out = ByteData.sublistView(digest);
  for (var i = 0; i < 8; i++) {
    out.setUint32(i * 4, state[i]);
  }
  return digest;
}
