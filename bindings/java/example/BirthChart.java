import static com.teispace.teistro.Civil.at;
import static com.teispace.teistro.Civil.date;
import static com.teispace.teistro.Civil.ianaZone;
import static com.teispace.teistro.Civil.whenUnknown;

import java.util.Arrays;
import java.util.Comparator;
import java.util.Locale;
import java.util.Map;
import java.util.stream.Collectors;

import com.teispace.teistro.Altitude;
import com.teispace.teistro.Calendar;
import com.teispace.teistro.CalendarDate;
import com.teispace.teistro.Chart;
import com.teispace.teistro.ChartKind;
import com.teispace.teistro.ChartOptions;
import com.teispace.teistro.CivilDateTime;
import com.teispace.teistro.Context;
import com.teispace.teistro.ContextOptions;
import com.teispace.teistro.Ephemeris;
import com.teispace.teistro.Json;
import com.teispace.teistro.Latitude;
import com.teispace.teistro.Longitude;
import com.teispace.teistro.Nakshatra;
import com.teispace.teistro.Observer;
import com.teispace.teistro.PlacedGraha;
import com.teispace.teistro.Rashi;
import com.teispace.teistro.Teistro;
import com.teispace.teistro.TeistroException;
import com.teispace.teistro.ZoneResolution;
import com.teispace.teistro.ZoneWarning;
import com.teispace.teistro.messages.Messages;

/**
 * A birth chart: from a Nepali birth record to the nine grahas placed.
 *
 * <p>This is the scenario the SDK exists for, and it is not one call. What
 * it takes, in order:
 *
 * <ol>
 *   <li>A birth record as people actually write one: a Bikram Sambat date,
 *       a local clock time, and a place.
 *   <li>That civil time resolved to an <b>instant</b>, which needs the zone's
 *       history: Nepal was +05:30 until 1986 and +05:45 after, and the
 *       resolution says which rule it used and from which tzdb.
 *   <li>The chart <b>founded</b> at that instant and place. The SDK's
 *       canonical frame is <i>tropical</i>, because that is what an ephemeris
 *       computes; a Vedic chart wants the sidereal zodiac, and the profile
 *       says which ayanamsha and which centre, so the founder asks for that
 *       frame and the SDK completes it, and stamps every step it applied,
 *       which this example prints. Positions asked for directly would not
 *       know the profile wants the Moon seen from Kathmandu rather than from
 *       the Earth's centre, which moves it most of a degree.
 *   <li>Each longitude read as a rashi, a nakshatra and a pada, using the
 *       catalogue's own members and the locale's own names.
 * </ol>
 *
 * <p>What it does <b>not</b> need: an ephemeris of your own, a data file, a
 * network, or a second library. {@code Ephemeris.BUILTIN} selects the one
 * the SDK carries, so every position below is a real sky and this file runs
 * anywhere the package installs.
 */
public final class BirthChart {
    /** A nakshatra is a twenty-seventh of the circle. */
    private static final double NAKSHATRA_DEG = 360.0 / 27.0;
    /** A pada is a quarter of a nakshatra. */
    private static final double PADA_DEG = NAKSHATRA_DEG / 4.0;

    private BirthChart() {}

    /** The sign a longitude stands in. */
    private static Rashi rashiOf(double longitude) {
        return Rashi.of((int) Math.floor(longitude / 30.0));
    }

    /** How far into its sign a longitude is. */
    private static double intoSign(double longitude) {
        return longitude % 30.0;
    }

    /** The lunar mansion a longitude stands in. */
    private static Nakshatra nakshatraOf(double longitude) {
        return Nakshatra.of((int) Math.floor(longitude / NAKSHATRA_DEG));
    }

    /** Which quarter of its lunar mansion a longitude stands in, 1 to 4. */
    private static int padaOf(double longitude) {
        return (int) Math.floor((longitude % NAKSHATRA_DEG) / PADA_DEG) + 1;
    }

    private static ContextOptions options(String settingsJson) {
        return ContextOptions.builder()
                .profile("nepali-default")
                .locale("ne-Deva-NP")
                .ephemeris(Ephemeris.BUILTIN)
                .settingsJson(settingsJson)
                .build();
    }

    /**
     * Runs the example.
     *
     * @param args unused
     */
    public static void main(String[] args) {
        Teistro teistro = Teistro.open();
        CalendarDate birthDay = date(Calendar.BIKRAM_SAMBAT, 2042, 9, 17);
        try (Context ctx = teistro.context(options(null))) {
            // ── 1. The record, as it would be written on a form ───────────
            CalendarDate gregorian = ctx.calendar().convert(birthDay, Calendar.GREGORIAN);
            System.out.printf(Locale.ROOT, "born  BS %d-%02d-%02d  (%d-%02d-%02d)  00:20  Kathmandu%n",
                    birthDay.year(), birthDay.month(), birthDay.day(),
                    gregorian.year(), gregorian.month(), gregorian.day());

            // ── 2. The instant, with the zone's own history ───────────────
            ZoneResolution when = ctx.time().resolve(at(birthDay, 0, 20), ianaZone("Asia/Kathmandu"));
            int offset = when.offsetSeconds();
            System.out.printf(Locale.ROOT, "      JD %.6f UTC   offset %+03d:%02d   %s (tzdb %s)%n",
                    when.instantJdUtc(), Math.floorDiv(offset, 3600), Math.abs(offset) % 3600 / 60,
                    when.source().key(), when.tzdbVersion());
            if (!when.timeKnown()) {
                System.out.println("      the time of day is not known; this chart is for noon");
            }
            // This record sits on the day Nepal moved from +05:30 to +05:45,
            // which is why the zone's history matters and a fixed offset
            // would be wrong: `IANA` above says the answer came from the
            // embedded database rather than from a guess.

            // ── 3. The chart ──────────────────────────────────────────────
            Observer place = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
            Chart chart = ctx.chart().found(when.instantJdUtc(), place, when.offsetSeconds(),
                    ChartOptions.builder().kind(ChartKind.NATAL).build());

            System.out.println();
            System.out.println("graha             sign               deg  nakshatra      pada bhava");
            System.out.println("─".repeat(67));
            for (PlacedGraha placed : chart.grahas()) {
                Messages.EntityForms graha = ctx.intl().entity(placed.graha().fullKey());
                double longitude = placed.longitudeDeg();
                // There is no retrograde flag to trust blindly: a graha is
                // retrograde when its longitude is decreasing, which is what
                // the speed says, and `retrograde()` is that comparison, named.
                String mark = placed.retrograde() ? "℞" : " ";
                // The bhava is the chart's placement system's answer -- the
                // question most of the tradition answers with "in the seventh".
                System.out.printf(Locale.ROOT, "%-12s %-2s %-1s %-12s %8.4f°  %-14s %d   %2d%n",
                        graha.name(), graha.glyph() == null ? "" : graha.glyph(), mark,
                        ctx.intl().entity(rashiOf(longitude).fullKey()).name(), intoSign(longitude),
                        ctx.intl().entity(nakshatraOf(longitude).fullKey()).name(), padaOf(longitude),
                        placed.house().bhava());
            }

            // ── 4. What the chart is measured in, and against ─────────────
            System.out.println();
            double lagna = chart.lagnaDeg();
            System.out.printf(Locale.ROOT, "lagna          %.4f° -- %s at %.4f°, vara %s%n",
                    lagna, ctx.intl().entity(rashiOf(lagna).fullKey()).name(), intoSign(lagna),
                    chart.day().vara().fullKey());
            // Empty, and it means what it says: a tropical chart has no
            // ayanamsha, not an ayanamsha of nought.
            String applied = chart.ayanamsha().map(a -> a.fullKey())
                    .orElse(chart.ayanamshaCustom() ? "custom" : null);
            if (applied == null) {
                System.out.println("ayanamsha      tropical, none applied");
            } else {
                System.out.printf(Locale.ROOT, "ayanamsha      %.6f° applied (%s)%n",
                        chart.ayanamshaOffsetDeg(), applied);
            }
            System.out.println("steps applied  " + String.join(", ", chart.batch().stepsApplied()));
            // The provenance envelope stamps the settings, the provider and
            // the time layer; it is what a stored chart keeps in order to say
            // what computed it.
            System.out.println("settings hash  " + ctx.settingsHash().substring(0, 16) + "…");
        }

        // ── A birth with no recorded time ─────────────────────────────────
        // The commonest data problem in the field, and the SDK does **not**
        // pick a time for you. `whenUnknown` says the time is unknown; what
        // happens next is the profile's `time.unknown_time` policy, and by
        // default there is none, so the call is refused with a hint naming
        // the choices.
        System.out.println();
        CivilDateTime noTime = whenUnknown(birthDay);
        for (String policy : Arrays.asList(null, "NOON", "MIDNIGHT")) {
            String settings = policy == null ? null : Json.write(Map.of("time", Map.of("unknown_time", policy)));
            try (Context scoped = teistro.context(options(settings))) {
                String label = String.format(Locale.ROOT, "%-9s", policy == null ? "refuse" : policy);
                try {
                    ZoneResolution resolved = scoped.time().resolve(noTime, ianaZone("Asia/Kathmandu"));
                    // The warnings are catalogue members here rather than
                    // strings, so the key is what to print; it is the same
                    // word in every binding.
                    String warnings = resolved.warnings().stream()
                            .sorted(Comparator.comparingInt(ZoneWarning::id))
                            .map(ZoneWarning::key)
                            .collect(Collectors.joining(", "));
                    System.out.printf(Locale.ROOT, "%s JD %.6f  time known %b  %s%n",
                            label, resolved.instantJdUtc(), resolved.timeKnown(),
                            warnings.isEmpty() ? "(no warning)" : warnings);
                } catch (TeistroException error) {
                    System.out.println(label + " " + error.getMessage());
                    System.out.println(" ".repeat(10) + "hint: " + error.hint());
                }
            }
        }
        // MIDNIGHT is refused for a different reason, and it is this record's
        // own: the clocks jumped at midnight on this very date, so 00:00
        // never happened in Kathmandu. A chart cast on a guessed midnight
        // would have been cast on a time that does not exist.
        // NOON answers, and says so twice -- `timeKnown()` is false and the
        // resolution carries a `TIME_UNKNOWN_FALLBACK` warning -- so a stored
        // chart can never quietly claim a birth time it never had.
    }
}
