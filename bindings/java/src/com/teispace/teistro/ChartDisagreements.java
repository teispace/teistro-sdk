package com.teispace.teistro;

import java.util.List;
import java.util.Map;

import com.teispace.teistro.record.Provenance;

/**
 * Where two charts disagree ({@code 03-design/chart-disagreements.md}): what
 * {@link ChartArea#disagreements} answers, only the facts that differ, each with both sides.
 *
 * <pre>{@code
 * ChartDisagreements found = classical.chart().disagreements(kp, new ComparedBirth(2_447_000.25, kathmandu, 20_700));
 * boolean sameLagna = found.lagna() == null;
 * }</pre>
 *
 * @param otherProfile the profile the other context resolved
 * @param otherSettingsHash the other context's settings hash, so a stored comparison is known stale when
 *     either side changes
 * @param at the instant the dasha lords were compared at; null for each birth's own
 * @param compared how many facts were compared
 * @param lagna the lagna, when its sign differs
 * @param grahas each graha whose sign differs
 * @param nakshatra the Moon's nakshatra, when it or its pada differs
 * @param dashas each dasha level whose running lord differs
 * @param yogas each rule one side holds and the other does not, or with another status
 * @param provenance the first chart's, which sealed the record; its input hash is the request's
 */
public record ChartDisagreements(String otherProfile, String otherSettingsHash, Double at, int compared,
        Placements lagna, List<GrahaPlacements> grahas, Stars nakshatra, List<Dasha> dashas, List<Yoga> yogas,
        Provenance provenance) {
    /**
     * A placement as one side reads it: the sign, and where in the circle and the sign it stands.
     *
     * @param sign the sign
     * @param longitudeDeg the longitude in that side's zodiac, degrees
     * @param degreesInSign degrees into the sign, 0 to 30
     */
    public record Placement(Rashi sign, double longitudeDeg, double degreesInSign) {}

    /**
     * The lagna in two signs.
     *
     * @param first as the first chart has it
     * @param second as the second has it
     */
    public record Placements(Placement first, Placement second) {}

    /**
     * A graha in two signs.
     *
     * @param graha which graha
     * @param first as the first chart places it
     * @param second as the second does
     */
    public record GrahaPlacements(Graha graha, Placement first, Placement second) {}

    /**
     * The Moon's star and pada as one side reads it.
     *
     * @param nakshatra the janma nakshatra
     * @param pada 1 to 4
     * @param longitudeDeg the Moon's longitude in that side's zodiac, degrees
     */
    public record Star(Nakshatra nakshatra, int pada, double longitudeDeg) {}

    /**
     * The Moon's nakshatra or pada, differing.
     *
     * @param first as the first chart has it
     * @param second as the second has it
     */
    public record Stars(Star first, Star second) {}

    /**
     * The period of one dasha level running at the instant compared.
     *
     * @param lord its lord
     * @param sign the sign, for a sign dasha; null for a graha's
     * @param interval its span
     */
    public record Running(Graha lord, Rashi sign, Interval interval) {}

    /**
     * One dasha level whose running lord differs; a side whose periods do not reach the instant is null.
     *
     * @param system the system's full key, registered or catalogued
     * @param level 1 for the mahadasha, 2 for the antardasha, and on
     * @param first the first chart's period
     * @param second the second's
     */
    public record Dasha(String system, int level, Running first, Running second) {}

    /**
     * A rule one side holds and the other does not, or with another net status.
     *
     * @param rule the rule's key
     * @param category its category
     * @param first {@code active}, {@code partially-cancelled} or {@code fully-cancelled} on the first chart;
     *     null where it does not hold
     * @param second the same on the second
     */
    public record Yoga(String rule, String category, String first, String second) {}

    /** The record {@code ts_chart_disagreements} wrote, its keys made members. */
    static ChartDisagreements read(String text) {
        Map<?, ?> envelope = Reads.object(Json.read(text));
        Map<?, ?> value = Reads.object(envelope, "value");
        Object lagna = Reads.field(value, "lagna");
        Object star = Reads.field(value, "nakshatra");
        return new ChartDisagreements(Reads.string(value, "otherProfile"), Reads.string(value, "otherSettingsHash"),
                Reads.optionalNumber(value, "at"), Reads.integer(value, "compared"),
                lagna == null ? null : new Placements(placement(Reads.object(lagna), "first"),
                        placement(Reads.object(lagna), "second")),
                Reads.each(value, "grahas", raw -> {
                    Map<?, ?> g = Reads.object(raw);
                    String graha = Reads.string(g, "graha");
                    return new GrahaPlacements(Reads.member(Graha.byKey(graha), "Graha", graha), placement(g, "first"),
                            placement(g, "second"));
                }),
                star == null ? null : new Stars(star(Reads.object(star), "first"), star(Reads.object(star), "second")),
                Reads.each(value, "dashas", raw -> {
                    Map<?, ?> d = Reads.object(raw);
                    return new Dasha(Reads.string(d, "system"), Reads.integer(d, "level"), running(d, "first"),
                            running(d, "second"));
                }),
                Reads.each(value, "yogas", raw -> {
                    Map<?, ?> y = Reads.object(raw);
                    return new Yoga(Reads.string(y, "rule"), Reads.string(y, "category"),
                            Reads.optionalString(y, "first"), Reads.optionalString(y, "second"));
                }),
                Provenance.of(Reads.field(envelope, "provenance")));
    }

    private static Placement placement(Map<?, ?> both, String side) {
        Map<?, ?> p = Reads.object(both, side);
        String sign = Reads.string(p, "sign");
        return new Placement(Reads.member(Rashi.byKey(sign), "Rashi", sign), Reads.number(p, "longitudeDeg"),
                Reads.number(p, "degreesInSign"));
    }

    private static Star star(Map<?, ?> both, String side) {
        Map<?, ?> s = Reads.object(both, side);
        String nakshatra = Reads.string(s, "nakshatra");
        return new Star(Reads.member(Nakshatra.byKey(nakshatra), "Nakshatra", nakshatra), Reads.integer(s, "pada"),
                Reads.number(s, "longitudeDeg"));
    }

    private static Running running(Map<?, ?> dasha, String side) {
        Object raw = Reads.field(dasha, side);
        if (raw == null) {
            return null;
        }
        Map<?, ?> r = Reads.object(raw);
        String lord = Reads.string(r, "lord");
        String sign = Reads.optionalString(r, "sign");
        return new Running(Reads.member(Graha.byKey(lord), "Graha", lord),
                sign == null ? null : Reads.member(Rashi.byKey(sign), "Rashi", sign),
                Reads.interval(Reads.field(r, "interval")));
    }
}
