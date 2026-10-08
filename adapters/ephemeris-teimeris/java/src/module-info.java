/**
 * The Teimeris ephemeris, as a Teistro adapter: the descriptor an
 * {@code ephemeris} chain takes, and the typed façade over the engine's own
 * operations.
 *
 * <p><b>AGPL-3.0-only</b>, because the library this module loads links
 * Teimeris. The SDK is Apache-2.0 and never links it: it loads the adapter
 * at run time, so the licence stays on this side of the boundary
 * (ADR-0029).
 */
module com.teispace.teistro.teimeris {
    requires transitive com.teispace.teistro;

    exports com.teispace.teistro.teimeris;
}
