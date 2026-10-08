package com.teispace.teistro;

/**
 * One graha's transit judged by the natal Ashtakavarga.
 *
 * @param graha Which graha.
 * @param bindus The bindus its own Ashtakavarga put in the sign it transits, 0 to 8 (v. 11),
 *     unreduced.
 * @param good Whether they reach {@code gochar.ashtakavarga_good_from}.
 * @param kakshya The eighth of the sign it stands in.
 * @param kakshyaBindu Whether that eighth's lord gave a bindu there, so that a bindu bears its
 *     fruit now.
 * @param sarva The sign's sarvashtakavarga.
 * @param sarvaStanding Where it stands against 28 (v. 20).
 */
public record AshtakavargaTransit(
        Graha graha,
        int bindus,
        boolean good,
        Kakshya kakshya,
        boolean kakshyaBindu,
        int sarva,
        SarvaStanding sarvaStanding) {
}
