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

    /**
     * The zone database this context reads ({@code 03-design/zone-packs.md}
     * §4): the release every answer's provenance names, and whether it came
     * from a signed pack or the build.
     *
     * @return the database
     */
    public ZoneDatabase zoneDatabase() {
        return ZoneDatabase.of(context.locked(Calls::timeZoneDatabase));
    }

    /**
     * Replaces this context's zone database with a signed pack, for every
     * answer after this one. A release under {@code atLeast}, the newest the
     * app has applied, is refused, so a replayed pack cannot roll the rules
     * back.
     *
     * @param pack the pack's exact bytes
     * @param signature the publisher's detached Ed25519 signature over them, 64 bytes
     * @param key the publisher's public key, 32 bytes, which the app fixes at its own build
     * @param atLeast the newest release applied; empty takes any
     * @return the database now read
     * @throws TeistroException {@code INVALID_ARG} naming {@code key}, {@code signature},
     *     {@code version}, {@code at_least} or {@code pack}; the database stays as it was
     */
    public ZoneDatabase loadZones(byte[] pack, byte[] signature, byte[] key, String atLeast) {
        byte[] bytes = pack.clone();
        byte[] signed = signature.clone();
        byte[] publisher = key.clone();
        return ZoneDatabase.of(context.locked(
                (lib, raw) -> Calls.timeLoadZones(lib, raw, bytes, signed, publisher, atLeast)));
    }
}
