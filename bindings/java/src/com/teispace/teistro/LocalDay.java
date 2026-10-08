package com.teispace.teistro;

/**
 * A local day, as a chart and an almanac both read it: the civil date, its weekday, the sunrise
 * that opened it, its sunset and the sunrise that closes it, whether it had a sunrise at all, and
 * by which convention. The same record in every binding.
 *
 * @param date The civil date, as {@code calendar.convert} returns one, so it can be handed back to
 *     it.
 * @param vara The weekday, which the sunrise-anchored reckoning keeps from sunrise to sunrise.
 * @param sunrise The sunrise that opened the day, or what the polar policy put in its place, as a
 *     Julian day (UTC).
 * @param sunset The sunset that closed its daylight, as a Julian day (UTC).
 * @param nextSunrise The sunrise that closes it, as a Julian day (UTC).
 * @param polar Null for a day the Sun rose and set on; what happened instead, for one it did not.
 *     May be null.
 * @param convention The named sunrise convention the day was reckoned by; null for a custom
 *     altitude. May be null.
 * @param customAltitudeDeg The custom altitude of the Sun's centre, degrees, when
 *     {@code convention} is null; null otherwise. May be null.
 * @param air The air the horizon was refracted through, resolved at the place, when the settings
 *     named one; null for the almanac's fixed 34 arcminutes or no refraction. May be null.
 */
public record LocalDay(
        CalendarDate date,
        Vara vara,
        double sunrise,
        double sunset,
        double nextSunrise,
        PolarDay polar,
        Sunrise convention,
        Double customAltitudeDeg,
        Air air) {
}
