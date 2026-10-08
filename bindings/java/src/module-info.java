/**
 * The Teistro SDK for Java: the library's C boundary through the Foreign
 * Function and Memory API, with no native code of its own.
 *
 * <p>A consumer grants the module native access, which binding the
 * library's entry points needs:
 * {@code --enable-native-access=com.teispace.teistro}.
 */
module com.teispace.teistro {
    exports com.teispace.teistro;
    exports com.teispace.teistro.blob;
    exports com.teispace.teistro.ffi;
}
