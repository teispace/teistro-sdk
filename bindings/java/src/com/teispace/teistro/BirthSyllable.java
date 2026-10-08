package com.teispace.teistro;

/**
 * The syllable a child is named by: the birth pada's own cell in the satapada cakra (C297 to
 * C299).
 *
 * @param cell Its place among the cakra's 112 cells, 0 for a, Krittika's first.
 * @param devanagari As <i>Muhurta Chintamani</i> p. 173 prints it.
 * @param iast Its IAST.
 * @param varga The letter group it begins in (VI.35).
 */
public record BirthSyllable(
        int cell,
        String devanagari,
        String iast,
        NameVarga varga) {
}
