package com.teispace.teistro;

/**
 * A graha the baseline's score names, with its house and verdict.
 *
 * @param graha the graha
 * @param house its house
 * @param verdict its verdict
 */
public record KeyInfluence(Graha graha, int house, GocharVerdict verdict) {}
