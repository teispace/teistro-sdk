package com.teispace.teistro;

import java.util.List;
import java.util.Optional;

/**
 * A Kamboola's grade: the Moon's offices and the pair's (Samjna Tantra vv. 36 to 46).
 *
 * @param rules The office rules it was graded under, every member filled.
 * @param moon The Moon's offices.
 * @param lagnesha The lagnesha's.
 * @param karyesha The karyesha's.
 * @param pair The lords' office when they hold the same one; null when they differ (C372). May be
 *     null.
 */
public record KamboolaGrade(
        com.teispace.teistro.request.OfficeRules rules,
        TajikaOffices moon,
        TajikaOffices lagnesha,
        TajikaOffices karyesha,
        String pair) {
    /**
     * The cell the gloss names, the Moon's office first and the pair's second; empty when the pair
     * has none.
     *
     * @return the two offices, or empty
     */
    public Optional<List<String>> cell() {
        return pair == null ? Optional.empty() : Optional.of(List.of(moon.adhikara(), pair));
    }
}
