package com.teispace.teistro;

/**
 * What a janma-patrika prints of the Moon: its star and pada, the syllable the child is named by,
 * and the readings the Ashta Koota takes of the same Moon (C301). Vashya, paya, disha and tatwa
 * are not here (C300).
 *
 * @param nakshatra The Moon's nakshatra.
 * @param pada Its pada, 1 to 4.
 * @param rashi The Moon's sign.
 * @param nakshatraLord The nakshatra's lord, the Vimshottari dasha's.
 * @param rashiLord The sign's lord, the one Graha Maitri reads.
 * @param varna The sign's varna, as Varna koota reads it (VI.22).
 * @param yoni The nakshatra's yoni.
 * @param gana The nakshatra's gana.
 * @param nadi The nakshatra's nadi.
 * @param syllable The syllable the child is named by.
 */
public record Avakahada(
        Nakshatra nakshatra,
        int pada,
        Rashi rashi,
        Graha nakshatraLord,
        Graha rashiLord,
        Varna varna,
        Yoni yoni,
        Gana gana,
        Nadi nadi,
        BirthSyllable syllable) {
}
