package com.teispace.teistro;

/**
 * One koota's points and what it read.
 *
 * @param points Its points, a multiple of a half.
 * @param maxPoints The most it gives, 1 for Varna to 8 for Nadi.
 * @param reading What it read.
 */
public record KootaRow(
        double points,
        double maxPoints,
        KootaReading reading) {
}
