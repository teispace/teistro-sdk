package com.teispace.teistro;

/**
 * Two planets past an aspect and still within their moieties (p. 110).
 *
 * @param aspect the aspect
 * @param pastDeg how far past exact, degrees
 */
public record Separation(PtolemaicAspect aspect, double pastDeg) {}
