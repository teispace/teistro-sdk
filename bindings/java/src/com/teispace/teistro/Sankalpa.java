package com.teispace.teistro;

/**
 * The time a sankalpa recites ({@code 03-design/sankalpa.md}): the year, ayana, ritu, month and
 * vara at the sunrise of the day holding the instant, and the limbs and three signs at the instant
 * itself.
 *
 * @param at the instant of the rite, a UTC Julian day
 * @param date the civil day whose sunrise the instant follows: a rite before dawn is the day
 *     before's
 * @param vara that day's weekday, which runs from its sunrise
 * @param sunrise that day's sunrise, a UTC Julian day
 * @param nextSunrise the next day's sunrise, where the day ends, a UTC Julian day
 * @param year the lunar year the day's sunrise falls in
 * @param ayana the half of the year at the day's sunrise
 * @param ayanaReckoning how {@code ayana} was reckoned, the request's: {@code SIDEREAL} or
 *     {@code TROPICAL}
 * @param ritu the season at the day's sunrise
 * @param rituReckoning how {@code ritu} was reckoned, {@code panchanga.ritu}: {@code SOLAR},
 *     {@code LUNAR} or {@code TROPICAL}
 * @param month the lunar month at the day's sunrise, under both conventions
 * @param paksha the fortnight of the tithi at the rite
 * @param tithi the tithi at the rite
 * @param nakshatra the nakshatra at the rite
 * @param yoga the yoga at the rite
 * @param karana the karana at the rite
 * @param sun the Sun's sidereal sign at the rite
 * @param moon the Moon's sidereal sign at the rite
 * @param jupiter Jupiter's sidereal sign at the rite
 */
public record Sankalpa(
        double at, CalendarDate date, Vara vara, double sunrise, double nextSunrise, LunarYear year,
        Ayana ayana, String ayanaReckoning, Ritu ritu, String rituReckoning, Month month, Paksha paksha,
        Tithi tithi, Nakshatra nakshatra, Yoga yoga, Karana karana, Rashi sun, Rashi moon, Rashi jupiter) {}
