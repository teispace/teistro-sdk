package com.teispace.teistro;

/**
 * The ishta-devata read from the amatyakaraka (vv. 76 to 79): the graha under the chart's chara
 * karaka scheme, its navamsha sign the 12th is counted from (C357), and the reading in both
 * charts.
 *
 * @param graha The amatyakaraka.
 * @param amsha Its navamsha sign.
 * @param inRasi The reading in the rasi chart.
 * @param inNavamsha The reading in the navamsha.
 */
public record AmatyaDevatas(
        Graha graha,
        Rashi amsha,
        AmatyaDevata inRasi,
        AmatyaDevata inNavamsha) {
}
