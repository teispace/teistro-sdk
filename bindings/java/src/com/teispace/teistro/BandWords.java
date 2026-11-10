package com.teispace.teistro;

import com.teispace.teistro.request.GunaBand;

/**
 * What {@link MatchingArea#band} answers: a guna total's band and the words
 * for it ({@code 03-design/matching.md}, "Grading a total into bands").
 *
 * @param total the total, out of 36
 * @param dushkuta whether the signs were taken to stand 2/12, 6/8 or 9/5
 * @param band the band
 * @param bands the table's name
 * @param row the row of the table that gave the band, from 0
 * @param locale the locale the words are in
 * @param words the band in words: the {@code sdk.matching.band} message
 * @param text the total with its band: the {@code sdk.matching.graded} message
 */
public record BandWords(double total, boolean dushkuta, GunaBand band, String bands, int row, String locale,
        String words, String text) {}
