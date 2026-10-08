package com.teispace.teistro;

/**
 * One graha's Vimshopaka, each score out of 20.
 *
 * @param graha Which graha, Sun to Saturn.
 * @param shadvarga Over the six vargas.
 * @param saptavarga Over the seven.
 * @param dashavarga Over the ten.
 * @param shodashavarga Over the sixteen.
 */
public record GrahaVimshopaka(
        Graha graha,
        double shadvarga,
        double saptavarga,
        double dashavarga,
        double shodashavarga) {
}
