package com.teispace.teistro;

/**
 * What stood at a death or a burning ({@code 03-design/death-time.md}).
 *
 * @param at the instant, a UTC Julian day
 * @param nakshatra the nakshatra at it
 * @param tithi the tithi at it
 * @param vara the vara of the sunrise day holding it
 * @param moon the Moon's sidereal sign at it
 * @param panchaka whether it is in the panchaka: the Moon in Aquarius or Pisces
 * @param feet {@code TRIPADA} or {@code DVIPADA}, when the nakshatra straddles a sign so; null otherwise
 * @param pushkara {@code TRIPUSHKARA} or {@code DVIPUSHKARA}, when one holds; null otherwise
 */
public record DeathInstant(double at, Nakshatra nakshatra, Tithi tithi, Vara vara, Rashi moon,
        boolean panchaka, String feet, String pushkara) {}
