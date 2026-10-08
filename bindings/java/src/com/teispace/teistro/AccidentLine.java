package com.teispace.teistro;

/**
 * One accidental line a planet meets, and what it scores for that planet:
 * orientality scores Saturn, Jupiter and Mars one way and Venus and
 * Mercury the other.
 *
 * @param accident the line
 * @param points what it scores
 */
public record AccidentLine(Accident accident, int points) {}
