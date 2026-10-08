package com.teispace.teistro;

/**
 * One graha's Vaiseshikamsa (BPHS ch. 6 vv. 42 to 53).
 *
 * @param graha Which graha, Sun to Saturn.
 * @param shadvarga Over the six vargas.
 * @param saptavarga Over the seven.
 * @param dashavarga Over the ten.
 * @param shodashavarga Over the sixteen.
 * @param impaired Whether it is combust, defeated in war or in Shayana, its names then not
 *     auspicious.
 */
public record GrahaVaiseshikamsa(
        Graha graha,
        VaiseshikamsaStanding shadvarga,
        VaiseshikamsaStanding saptavarga,
        VaiseshikamsaStanding dashavarga,
        VaiseshikamsaStanding shodashavarga,
        boolean impaired) {
}
