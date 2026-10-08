package com.teispace.teistro;

/**
 * What stops or hinders the application (pp. 110–113).
 *
 * @param kind which impediment
 * @param significator the significator it falls on
 * @param third the third planet; may be null, for a refranation
 * @param aspect the aspect the third perfects, or the one refrained from
 * @param days days until the contact, or the station
 */
public record Impediment(ImpedimentKind kind, Graha significator, Graha third, PtolemaicAspect aspect, double days) {}
