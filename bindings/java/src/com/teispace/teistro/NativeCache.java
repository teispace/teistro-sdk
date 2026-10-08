package com.teispace.teistro;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.file.FileAlreadyExistsException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.nio.file.attribute.PosixFilePermission;
import java.nio.file.attribute.PosixFilePermissions;
import java.security.DigestInputStream;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.HexFormat;
import java.util.Set;

/**
 * Where a library packaged in the jar is written so the platform's loader
 * can map it: a directory named by the version and the library's digest,
 * so every process and thread that needs that library converges on one
 * file.
 *
 * <p>The library is hashed as it is written and again every time it is
 * found, and on POSIX the directory must be this user's alone: under a
 * shared temporary directory a predictable path is one another user could
 * fill first.
 */
final class NativeCache {
    private NativeCache() {}

    /** The system property naming the directory the cache lives under. */
    static final String CACHE_PROPERTY = "teistro.cache";

    private static final Set<PosixFilePermission> PRIVATE = PosixFilePermissions.fromString("rwx------");

    /** The directory the cache lives under: {@value #CACHE_PROPERTY}, else {@code java.io.tmpdir}. */
    static Path root() {
        String named = System.getProperty(CACHE_PROPERTY);
        return Path.of(named != null && !named.isEmpty() ? named : System.getProperty("java.io.tmpdir"));
    }

    /**
     * The cached library for a digest, written from {@code library} when
     * it is not there yet.
     *
     * @param library the library's bytes, read only when they must be written
     * @param digest its SHA-256, lower-case hex, as the release staged it
     * @param root the directory the cache lives under
     * @param version the SDK's version
     * @param fileName the platform's file name for the library
     * @return the cached file
     * @throws IOException when the directory or file cannot be made, or the
     *     bytes do not hash to {@code digest}
     */
    static Path extract(InputStream library, String digest, Path root, String version, String fileName)
            throws IOException {
        Path dir = root.resolve("teistro-" + version + "-" + digest);
        if (Files.isDirectory(dir)) {
            owned(dir);
        } else {
            create(dir);
        }
        Path target = dir.resolve(fileName);
        if (Files.isRegularFile(target)) {
            String found = sha256(target);
            if (found.equals(digest)) {
                return target;
            }
            throw new IOException("the cached library " + target + " hashes to " + found + ", not the "
                    + digest + " it was staged with: delete it, and it is written again");
        }
        Path part = Files.createTempFile(dir, fileName, ".part");
        try {
            String written;
            try (DigestInputStream in = new DigestInputStream(library, sha256());
                    OutputStream out = Files.newOutputStream(part)) {
                in.transferTo(out);
                written = HexFormat.of().formatHex(in.getMessageDigest().digest());
            }
            if (!written.equals(digest)) {
                throw new IOException("the jar's library hashes to " + written + ", not the " + digest
                        + " it was staged with");
            }
            if (posix(dir)) {
                Files.setPosixFilePermissions(part, PosixFilePermissions.fromString("r-x------"));
            }
            try {
                Files.move(part, target, StandardCopyOption.ATOMIC_MOVE);
            } catch (IOException raced) {
                // Another process wrote it first; Windows will not replace a
                // library another process has mapped. Either way the file
                // there is used if it is the one staged.
                if (!(Files.isRegularFile(target) && sha256(target).equals(digest))) {
                    throw raced;
                }
            }
            return target;
        } finally {
            Files.deleteIfExists(part);
        }
    }

    private static void create(Path dir) throws IOException {
        Files.createDirectories(dir.getParent());
        try {
            if (posix(dir.getParent())) {
                Files.createDirectory(dir, PosixFilePermissions.asFileAttribute(PRIVATE));
            } else {
                Files.createDirectory(dir);
            }
        } catch (FileAlreadyExistsException raced) {
            owned(dir);
        }
    }

    /** Refuses a POSIX directory another user owns or may write to. */
    private static void owned(Path dir) throws IOException {
        if (!posix(dir)) {
            return;
        }
        String owner = Files.getOwner(dir).getName();
        String user = System.getProperty("user.name");
        Set<PosixFilePermission> mode = Files.getPosixFilePermissions(dir);
        if (!owner.equals(user) || !PRIVATE.containsAll(mode)) {
            throw new IOException("the library cache " + dir + " is " + owner + "'s with "
                    + PosixFilePermissions.toString(mode) + ", not " + user
                    + "'s alone; set -D" + CACHE_PROPERTY + " to a directory that is yours");
        }
    }

    private static boolean posix(Path path) {
        return path.getFileSystem().supportedFileAttributeViews().contains("posix");
    }

    static String sha256(Path file) throws IOException {
        try (DigestInputStream in = new DigestInputStream(Files.newInputStream(file), sha256())) {
            in.transferTo(OutputStream.nullOutputStream());
            return HexFormat.of().formatHex(in.getMessageDigest().digest());
        }
    }

    private static MessageDigest sha256() {
        try {
            return MessageDigest.getInstance("SHA-256");
        } catch (NoSuchAlgorithmException absent) {
            throw new IllegalStateException("every JVM provides SHA-256", absent);
        }
    }
}
