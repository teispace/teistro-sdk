// Analyses the generated surface, proves the wrong usages are analysis
// errors, and runs the runtime check. Prints one JSON line.
import 'dart:io';

void main() {
  final good = Process.runSync('dart', ['analyze', '--fatal-infos', 'lib', 'bin']);
  if (good.exitCode != 0) {
    stderr.write(good.stdout);
    exit(1);
  }
  final wrong = Process.runSync('dart', ['analyze', 'wrong']);
  final errors = RegExp(r'^\s*error -', multiLine: true)
      .allMatches(wrong.stdout.toString())
      .length;
  if (errors != 5) {
    stderr.write(wrong.stdout);
    exit(1);
  }
  final check = Process.runSync('dart', ['run', 'bin/check.dart']);
  if (check.exitCode != 0) {
    stderr.write(check.stdout);
    stderr.write(check.stderr);
    exit(1);
  }
  final calls = check.stdout.toString().trim();
  print('{"analyze":"ok","wrongUsagesRejected":$errors,"runtimeCalls":$calls}');
}
