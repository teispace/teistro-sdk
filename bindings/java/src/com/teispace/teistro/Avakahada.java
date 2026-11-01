package com.teispace.teistro;

/**
 * What a janma-patrika prints of the Moon: its star and pada, the syllable the child is named by,
 * the readings the Ashta Koota takes of the same Moon (C301), and the birth paya by both methods
 * (C1105 to C1107). Vashya, disha and tatwa are not here (C300).
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
 * @param ghata The ghata chakra of the Moon's sign: what strikes the native (C758).
 * @param payaByNakshatra The birth paya by the Moon's nakshatra (C1106).
 * @param payaByHouse The birth paya by the Moon's house counted from the lagna's sign (C1105, C1107).
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
        BirthSyllable syllable,
        Ghata ghata,
        Paya payaByNakshatra,
        Paya payaByHouse) {
}
