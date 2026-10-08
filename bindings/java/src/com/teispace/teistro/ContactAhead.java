package com.teispace.teistro;

/**
 * A significator's application to a collector.
 *
 * @param aspect the aspect
 * @param days days until it is exact
 */
public record ContactAhead(PtolemaicAspect aspect, double days) {}
