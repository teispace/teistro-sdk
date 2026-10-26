package com.teispace.teistro;

/**
 * What strikes a native of one Moon sign: Muhurta Chintamani's four rows of the ghata chakra
 * (XI vv. 27–30, C758).
 *
 * @param chandra the sign the Moon strikes from (v. 27)
 * @param tithi the class of tithi that strikes (v. 28)
 * @param vara the weekday that strikes (v. 29)
 * @param nakshatra the nakshatra that strikes (v. 30)
 */
public record Ghata(Rashi chandra, TithiClass tithi, Vara vara, Nakshatra nakshatra) {
}
