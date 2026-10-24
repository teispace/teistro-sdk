// The binding gives back the native memory it takes
// (`docs/05-testing/01-quality-bar.md`, "leak checks per binding"), read as
// the Python binding's `tests/test_leaks.py` reads it: the C allocator's own
// count of the bytes in use, `malloc_zone_statistics` on macOS and glibc's
// `mallinfo2` on Linux. The resident set will not do, since macOS compresses
// idle pages and a process keeping its contexts shrinks.
//
// Each family runs its cycles once to settle, then `runs` times measured, and
// at least one measured run must leave the bytes in use within `budget` of
// where it began: a leak grows them in every run, while what the VM allocates
// for itself moves them a little, and now and then once. Unread on Windows,
// on musl, and on Intel macOS, whose zones' `size_in_use` is not the bytes
// live (the Python test says why).
//
// A context dropped undisposed is freed by its `NativeFinalizer` when the VM
// collects it, and a test cannot make the VM collect, so that path is held
// by the Python and Java tests over the same C free, not here.
//
// Tagged `leaks`, so it runs alone (`dart_test.yaml`): another file run
// beside it allocates from the same allocator.
@Tags(['leaks'])
library;

import 'dart:ffi';

import 'package:ffi/ffi.dart';
import 'package:teistro/teistro.dart';
import 'package:test/test.dart';

/// How far the bytes in use may move over a measured run.
const budget = 64 * 1024;

/// The cycles of one run.
const cycles = 500;

/// The measured runs of each family.
const runs = 3;

/// macOS's `malloc_statistics_t`.
final class _ZoneStatistics extends Struct {
  @UnsignedInt()
  external int blocksInUse;
  @Size()
  external int sizeInUse;
  @Size()
  external int maxSizeInUse;
  @Size()
  external int sizeAllocated;
}

/// glibc's `struct mallinfo2`.
final class _Mallinfo2 extends Struct {
  @Size()
  external int arena;
  @Size()
  external int ordblks;
  @Size()
  external int smblks;
  @Size()
  external int hblks;
  @Size()
  external int hblkhd;
  @Size()
  external int usmblks;
  @Size()
  external int fsmblks;
  @Size()
  external int uordblks;
  @Size()
  external int fordblks;
  @Size()
  external int keepcost;
}

/// How to read the bytes in use, where the allocator says.
int Function()? _reader() {
  final process = DynamicLibrary.process();
  switch (Abi.current()) {
    case Abi.macosArm64:
      final statistics = process.lookupFunction<
        Void Function(Pointer<Void>, Pointer<_ZoneStatistics>),
        void Function(Pointer<Void>, Pointer<_ZoneStatistics>)
      >('malloc_zone_statistics');
      // Allocated once, so reading allocates nothing it then counts.
      final stats = calloc<_ZoneStatistics>();
      return () {
        statistics(nullptr, stats);
        return stats.ref.sizeInUse;
      };
    case Abi.linuxX64 || Abi.linuxArm64:
      if (!process.providesSymbol('mallinfo2')) {
        return null;
      }
      final mallinfo2 = process
          .lookupFunction<_Mallinfo2 Function(), _Mallinfo2 Function()>(
            'mallinfo2',
          );
      return () {
        final info = mallinfo2();
        // The small blocks in use and the large ones mapped on their own.
        return info.uordblks + info.hblkhd;
      };
    default:
      return null;
  }
}

final int Function()? inUse = _reader();

/// At least one of [runs] runs of [cycle] grows the bytes in use within
/// [budget]. A failure names every run's growth and one run's over four
/// times the cycles: a leak grows with the cycles, and what the allocator
/// keeps for itself does not.
void givesBack(void Function() cycle) {
  final read = inUse!;
  int run(int count) {
    final before = read();
    for (var at = 0; at < count; at += 1) {
      cycle();
    }
    return read() - before;
  }

  run(cycles);
  final grown = [for (var at = 0; at < runs; at += 1) run(cycles)];
  final least = grown.reduce((a, b) => a < b ? a : b);
  if (least <= budget) {
    return;
  }
  final longer = run(4 * cycles);
  fail(
    'the bytes in use grew $grown over $runs runs of $cycles cycles '
    '(${least ~/ cycles} a cycle at least), and $longer over ${4 * cycles}; '
    'the budget is $budget',
  );
}

void main() {
  final teistro = Teistro.open();
  final skip =
      inUse == null
          ? "the allocator's bytes in use cannot be read faithfully here"
          : null;

  test('a context disposed gives back its native memory', skip: skip, () {
    givesBack(() {
      final ctx = teistro.context(
        profile: 'nepali-default',
        testProvider: true,
      );
      expect(ctx.profile, isNotEmpty);
      ctx.dispose();
    });
  });

  test('charts and positions give back their answers', skip: skip, () {
    final ctx = teistro.context(
      profile: 'nepali-default',
      locale: 'en-Latn',
      ephemeris: const [NamedEphemeris(Ephemeris.builtin)],
    );
    final kathmandu = Observer(
      latitudeDeg: Latitude(27.7172),
      longitudeDeg: Longitude(85.324),
      altitudeM: Altitude(1400),
    );
    try {
      givesBack(() {
        ctx.chart.found(
          instant: 2451545.0,
          place: kathmandu,
          utcOffsetSeconds: 20700,
        );
        ctx.positions(
          instants: [2451545.0, 2451546.0],
          bodies: [Body.sun, Body.moon],
        );
      });
    } finally {
      ctx.dispose();
    }
  });
}
