package com.teispace.teistro;

import java.util.Optional;

/**
 * A day's times' ± band from Delta T (SR-104, {@code 03-design/time-and-timezone.md} §3.5): the Delta T the day was
 * reckoned under and the one-sigma band, in seconds, each kind of time takes on from its uncertainty. Each band is
 * empty when Delta T states none.
 *
 * @param deltaTSeconds Delta T, TT less UT1, at the middle of the day's window, seconds
 * @param band where its uncertainty comes from
 * @param limbsSeconds a limb's end, a sankranti or a lunation: Delta T's whole band
 * @param sunriseSeconds the sunrise and sunset and every period divided from them: a 365th of it
 * @param moonriseSeconds the moonrise and moonset: a 26th of it
 */
public record TimesUncertainty(double deltaTSeconds, DeltaTBand band, Optional<Double> limbsSeconds,
        Optional<Double> sunriseSeconds, Optional<Double> moonriseSeconds) {}
