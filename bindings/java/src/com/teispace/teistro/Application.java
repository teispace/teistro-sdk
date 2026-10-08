package com.teispace.teistro;

/**
 * The significators coming to an aspect (p. 107).
 *
 * @param aspect the aspect
 * @param days days until it is exact
 * @param applying the significator whose motion closes it
 * @param kind how it applies
 * @param gapDeg how far it is from exact now, degrees
 * @param withinMoieties whether the gap is already within the two planets'
 *     moieties of orb
 */
public record Application(
        PtolemaicAspect aspect, double days, Graha applying, ApplicationKind kind, double gapDeg,
        boolean withinMoieties) {}
