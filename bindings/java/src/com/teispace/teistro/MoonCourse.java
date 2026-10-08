package com.teispace.teistro;

/**
 * Where the Moon is going before she leaves her sign (p. 112).
 *
 * @param next her first perfection before she leaves her sign; may be null,
 *     when void by that reading
 * @param withinOrb the first already within the two planets' moieties; may
 *     be null, when void by Lilly's moieties (C230)
 * @param daysInSign days before she leaves her sign
 * @param eased Taurus, Cancer, Sagittarius or Pisces, where void "somewhat
 *     she performes"
 */
public record MoonCourse(Perfection next, Perfection withinOrb, double daysInSign, boolean eased) {}
