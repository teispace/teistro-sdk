package com.teispace.teistro;

/**
 * One marriage dosha a match carries (C289). No severity is given, since no text grades one
 * (C290).
 *
 * @param system The reading it comes from.
 * @param koota The koota or consideration it is; null for the Kuja dosha, which is no koota. May
 *     be null.
 * @param side The side carrying it, for the Kuja dosha; null for the rest. May be null.
 * @param lifted Whether an exception the source names lifts it.
 */
public record MarriageDosha(
        DoshaSystem system,
        Koota koota,
        MatchRole side,
        boolean lifted) {
}
