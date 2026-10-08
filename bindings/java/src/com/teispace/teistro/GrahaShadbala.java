package com.teispace.teistro;

/**
 * One graha's Shadbala, in virupas.
 *
 * @param graha Which graha, Sun to Saturn.
 * @param sthana Positional strength by component.
 * @param dig Directional strength, 0 to 60.
 * @param kaala Temporal strength by component.
 * @param cheshta Motional strength.
 * @param naisargika Natural strength.
 * @param drik Aspectual strength, which may be negative.
 * @param virupas The six together.
 * @param rupas The six together, in rupas.
 * @param requiredRupas The rupas it must reach to be strong.
 * @param strong Whether it reaches them.
 * @param ishta How far it tends to good, 0 to 60 (BPHS ch. 28).
 * @param kashta How far it tends to harm, 0 to 60.
 * @param subhaRashmi Its auspicious rays, 1 to 7: the mean of its Uchcha and Cheshta rays (BPHS
 *     ch. 28 v. 5).
 * @param ashubhaRashmi Its inauspicious rays, 8 less the auspicious.
 */
public record GrahaShadbala(
        Graha graha,
        SthanaBala sthana,
        double dig,
        KaalaBala kaala,
        double cheshta,
        double naisargika,
        double drik,
        double virupas,
        double rupas,
        double requiredRupas,
        boolean strong,
        double ishta,
        double kashta,
        double subhaRashmi,
        double ashubhaRashmi) {
}
