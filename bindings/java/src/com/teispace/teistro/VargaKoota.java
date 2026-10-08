package com.teispace.teistro;

/**
 * Two names' vargas and how they stand (<i>Muhurta Chintamani</i> VI.35, C295).
 *
 * @param bride the bride's name's varga
 * @param groom the groom's name's varga
 * @param relation one varga, enemies (each the 5th from the other), or neither
 */
public record VargaKoota(NameVarga bride, NameVarga groom, VargaRelation relation) {}
