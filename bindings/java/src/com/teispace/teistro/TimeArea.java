package com.teispace.teistro;

/** {@code sky.time()}: the scales, the zones and what separates them. */
public final class TimeArea {
    private final Context context;

    TimeArea(Context context) {
        this.context = context;
    }

    /**
     * The instant a civil date and time in a zone stands for.
     *
     * @param civil the civil date and time
     * @param zone the zone it is read in
     * @return the instant, and how the zone resolved it
     */
    public ZoneResolution resolve(CivilDateTime civil, ZoneSpec zone) {
        return context.locked((lib, raw) -> Calls.timeResolve(lib, raw, civil, zone));
    }

    /**
     * The civil date and time an instant is, in a zone.
     *
     * @param jdUtc the instant as a UTC Julian day
     * @param zone the zone
     * @param calendar the calendar the date is given in
     * @return the civil date and time, and how the zone resolved it
     */
    public TimeCivilResult civilOf(double jdUtc, ZoneSpec zone, Calendar calendar) {
        return context.locked((lib, raw) -> Calls.timeCivil(lib, raw, jdUtc, zone, calendar));
    }

    /**
     * The same instant on another time scale. {@link Scale} and not
     * {@link TimeScale}: the time layer knows UTC as well as the two the port
     * carries.
     *
     * @param jd the instant as a Julian day on {@code scale}
     * @param scale the scale it is on
     * @param into the scale to express it on
     * @return the instant on {@code into}
     */
    public TimeConversion convert(double jd, Scale scale, Scale into) {
        return context.locked((lib, raw) -> Calls.timeConvert(lib, raw, jd, scale, into));
    }

    /**
     * TT less UT1 at an instant, and where the value came from.
     *
     * @param jdUt1 the instant as a UT1 Julian day
     * @return delta T
     */
    public DeltaT deltaT(double jdUt1) {
        return context.locked((lib, raw) -> Calls.timeDeltaT(lib, raw, jdUt1));
    }
}
