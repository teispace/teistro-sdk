package com.teispace.teistro;

/**
 * Saturn's house from a sign, its Sade Sati phase there, and whether the
 * house is one of the smaller spells.
 *
 * @param house Saturn's house from the sign
 * @param sadeSati {@code RISING} (the 12th), {@code PEAK} (the 1st) or
 *     {@code SETTING} (the 2nd); may be null, outside Sade Sati
 * @param spell whether the house is one of the smaller spells
 */
public record SaturnStanding(int house, String sadeSati, boolean spell) {}
