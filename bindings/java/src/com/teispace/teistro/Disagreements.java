package com.teispace.teistro;

import java.util.List;

import com.teispace.teistro.record.Provenance;

/**
 * Where two reckonings of one range disagree, and why ({@code 03-design/setup-disagreements.md}): what
 * {@link AlmanacArea#disagreements} answers, each comparison asked, null when not.
 *
 * <pre>{@code
 * Disagreements found = official.almanac().disagreements(modern, first, last, kathmandu, 20_700,
 *         true, new FestivalRequest("NEPAL"));
 * double moved = found.days().days().get(0).tithi().official().change().minutesFromSunrise();
 * }</pre>
 *
 * @param days the days, when compared
 * @param festivals the festivals, when asked
 */
public record Disagreements(Days days, Festivals festivals) {
    /**
     * The instant one reckoning's member gives way to the other's, under one sky.
     *
     * @param at the instant, a Julian day (UT)
     * @param minutesFromSunrise minutes from that sky's sunrise to it, negative before sunrise
     */
    public record Change(double at, double minutesFromSunrise) {}

    /**
     * One limb of one day as one reckoning names it at its sunrise.
     *
     * @param <T> the limb's members
     * @param member the member standing at this reckoning's sunrise
     * @param span that member's whole span under this reckoning's sky
     * @param sunrise this reckoning's sunrise, a Julian day (UT)
     * @param change where the two members meet under this sky, or null when they are not neighbours
     */
    public record Side<T>(T member, Interval span, double sunrise, Change change) {}

    /**
     * One limb both reckonings name differently at sunrise.
     *
     * @param <T> the limb's members
     * @param official as this context reckons it: the day the app shows
     * @param other as the context compared with reckons it
     */
    public record Limb<T>(Side<T> official, Side<T> other) {}

    /**
     * A day on which some limb's member at sunrise differs; a limb both name alike is null.
     *
     * @param date the civil day
     * @param tithi the tithi, when it differs
     * @param nakshatra the nakshatra, when it differs
     * @param yoga the nitya yoga, when it differs
     * @param karana the karana, when it differs
     */
    public record Day(CalendarDate date, Limb<Tithi> tithi, Limb<Nakshatra> nakshatra, Limb<Yoga> yoga,
            Limb<Karana> karana) {}

    /**
     * The days two reckonings disagree on, and what the other one was.
     *
     * @param otherProfile the profile the other context resolved
     * @param otherSettingsHash the other context's settings hash, so a stored comparison is known stale
     *     when either side's settings change
     * @param compared how many days were compared
     * @param days the days that differ, in order
     * @param provenance this context's, which sealed the record
     */
    public record Days(String otherProfile, String otherSettingsHash, int compared, List<Day> days,
            Provenance provenance) {
        /** Keeps the list unmodifiable. */
        public Days {
            days = List.copyOf(days);
        }
    }

    /**
     * One rule's occurrence the two reckonings give differently, or one alone gives.
     *
     * @param rule the rule's key
     * @param month its amanta month as this context names it, or as the other does when only it gives one
     * @param adhika whether that month is adhika, the same way
     * @param official this context's observance, with each day's extent of the rite's window, which is
     *     the reason; null when it gives none
     * @param other the other's
     */
    public record Festival(String rule, Masa month, boolean adhika, FestivalObservance official,
            FestivalObservance other) {}

    /**
     * One Ekadashi the two reckonings fast on differently, or one alone fasts.
     *
     * @param rule the rule's key
     * @param tithi the Ekadashi
     * @param month its amanta month, as {@link Festival#month} is
     * @param adhika whether that month is adhika
     * @param official this context's fast, with the vedha and excess that decided; null when it gives none
     * @param other the other's
     */
    public record Ekadashi(String rule, Tithi tithi, Masa month, boolean adhika, EkadashiFast official,
            EkadashiFast other) {}

    /**
     * The festivals two reckonings disagree on.
     *
     * @param otherProfile the profile the other context resolved
     * @param otherSettingsHash the other context's settings hash
     * @param compared how many observances and fasts this context gave
     * @param observances the observances that differ
     * @param ekadashis the Ekadashi fasts that differ
     * @param provenance this context's, which sealed the record
     */
    public record Festivals(String otherProfile, String otherSettingsHash, int compared,
            List<Festival> observances, List<Ekadashi> ekadashis, Provenance provenance) {
        /** Keeps the lists unmodifiable. */
        public Festivals {
            observances = List.copyOf(observances);
            ekadashis = List.copyOf(ekadashis);
        }
    }
}
