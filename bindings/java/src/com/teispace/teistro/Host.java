package com.teispace.teistro;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Locale;

/**
 * The host as a release names it: the platform whose library this
 * process can load, and the file name the platform's loader gives it.
 *
 * <p>Pure functions over the values a JVM reports, so a test feeds it the
 * hosts no machine of ours runs.
 */
final class Host {
    private Host() {}

    /** The system property that names the platform outright, for a host the detection misreads. */
    static final String PLATFORM_PROPERTY = "teistro.platform";

    /**
     * This process's platform: {@value #PLATFORM_PROPERTY} when set, else
     * as {@code os.name} and {@code os.arch} report it, with musl found
     * from the process's own loader.
     */
    static String platform() {
        String named = System.getProperty(PLATFORM_PROPERTY);
        if (named != null && !named.isEmpty()) {
            return named;
        }
        String os = System.getProperty("os.name", "");
        String arch = System.getProperty("os.arch", "");
        return platform(os, arch, os.startsWith("Linux") && musl(arch));
    }

    /**
     * The platform name for an {@code os.name}, an {@code os.arch} and
     * whether the C library is musl. An x64 JVM under Rosetta or Windows
     * on Arm reports x64, which is right: the library must match the
     * process, not the machine.
     */
    static String platform(String osName, String osArch, boolean musl) {
        String os;
        if (osName.startsWith("Windows")) {
            os = "win32";
        } else if (osName.startsWith("Mac")) {
            os = "darwin";
        } else if (osName.startsWith("Linux")) {
            os = "linux";
        } else {
            os = osName.toLowerCase(Locale.ROOT).replace(' ', '-');
        }
        String cpu = switch (osArch) {
            case "amd64", "x86_64" -> "x64";
            case "aarch64", "arm64" -> "arm64";
            default -> osArch;
        };
        return os + "-" + cpu + (os.equals("linux") && musl ? "-musl" : "");
    }

    /**
     * Whether this Linux process runs on musl: its own loader, read from
     * {@code /proc/self/maps}, so a glibc host with musl's tools installed
     * is not misread; failing that, musl's loader on disk.
     */
    static boolean musl(String osArch) {
        try {
            for (String line : Files.readAllLines(Path.of("/proc/self/maps"))) {
                if (line.contains("/ld-musl-")) {
                    return true;
                }
            }
            return false;
        } catch (IOException | SecurityException unreadable) {
            String machine = osArch.equals("aarch64") || osArch.equals("arm64") ? "aarch64" : "x86_64";
            return Files.exists(Path.of("/lib/ld-musl-" + machine + ".so.1"));
        }
    }

    /** The file name this platform gives the shared library. */
    static String fileName() {
        return fileName(System.getProperty("os.name", ""));
    }

    /** The file name an {@code os.name}'s platform gives the shared library. */
    static String fileName(String osName) {
        if (osName.startsWith("Windows")) {
            return "teistro_ffi.dll";
        }
        return osName.startsWith("Mac") ? "libteistro_ffi.dylib" : "libteistro_ffi.so";
    }
}
