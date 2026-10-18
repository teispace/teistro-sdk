package com.teispace.teistro;

import com.teispace.teistro.request.TithiRule;

/**
 * One year's tithi pravesha ({@code 03-design/tithi-pravesha.md}): the instant the Moon's
 * elongation from the Sun returns to the birth's, the solar return it was chosen around and which
 * rule chose it. Found the year's chart at {@link #at()}, where the native is.
 *
 * @param year The year, {@code 1} the first return, a year after birth, as a Varsha Pravesha's
 *     year counts it.
 * @param at The instant the elongation returns, a Julian day (UTC).
 * @param solarReturn The year's sidereal solar return, a Julian day (UTC): what "nearest" is
 *     measured from.
 * @param elongationDeg The birth's elongation returned to, the Moon less the Sun, degrees in
 *     {@code [0, 360)}.
 * @param rule The rule asked for.
 * @param picked The rule that chose this instant: {@code NEAREST_RETURN} where {@code NATAL_SIGN}
 *     found no phase return in the sign (C446).
 * @param inSign How many phase returns fell while the Sun was in its natal sign: 0, 1 or 2.
 */
public record TithiPravesha(
        int year,
        double at,
        double solarReturn,
        double elongationDeg,
        TithiRule rule,
        TithiRule picked,
        int inSign) {
    /**
     * Whether the year fell back: {@link #rule()} and {@link #picked()} differ.
     *
     * @return whether the natal sign held no phase return and the nearest was taken
     */
    public boolean fellBack() {
        return rule != picked;
    }
}
