package com.teispace.teistro;

/**
 * The ishta-devata, in the rasi chart and in the navamsha, and the same read from the amatya.
 *
 * @param atmakaraka The atmakaraka.
 * @param karakamsha The karakamsha.
 * @param inRasi The reading in the rasi chart.
 * @param inNavamsha The reading in the navamsha.
 * @param amatya The same read from the amatya.
 */
public record IshtaDevatas(
        Graha atmakaraka,
        Rashi karakamsha,
        IshtaDevata inRasi,
        IshtaDevata inNavamsha,
        AmatyaDevatas amatya) {
}
