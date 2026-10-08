import static com.teispace.teistro.Civil.date;

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Optional;

import com.teispace.teistro.Abhijit;
import com.teispace.teistro.AlmanacDay;
import com.teispace.teistro.Altitude;
import com.teispace.teistro.Calendar;
import com.teispace.teistro.Catalogued;
import com.teispace.teistro.ChoghadiyaPeriod;
import com.teispace.teistro.Context;
import com.teispace.teistro.ContextOptions;
import com.teispace.teistro.Ephemeris;
import com.teispace.teistro.KaalaPeriod;
import com.teispace.teistro.Latitude;
import com.teispace.teistro.Longitude;
import com.teispace.teistro.MoonEventMoment;
import com.teispace.teistro.Observer;
import com.teispace.teistro.Span;
import com.teispace.teistro.Teistro;
import com.teispace.teistro.TeistroException;
import com.teispace.teistro.blob.Day;

/**
 * A week's panchangam: the five limbs of each day, and its periods.
 *
 * <p>A chart is consulted once; a panchanga every morning. This is the page
 * a Nepali or Indian almanac prints, and the SDK computes it in <b>one
 * crossing</b> for the whole week: consecutive days share a boundary, so
 * day n's next sunrise is day n+1's sunrise, and asking for seven days
 * costs much less than seven days asked for separately.
 *
 * <p>What the shape teaches, and what a reader should copy:
 *
 * <ul>
 *   <li>A limb is a <b>span</b>, not a name. "Today's tithi" is a question
 *       with two answers on most days, and the SDK gives both with the
 *       instant each gives way, which is what an almanac row prints.
 *   <li>A span carries its <b>own</b> bounds as well as the clipped ones,
 *       so "the tithi began yesterday at 21:05" is a fact you can print.
 *   <li>A value a day may not have is <b>absent</b>, never a sentinel: no
 *       sankranti is an empty {@code Optional}, not Julian day zero.
 * </ul>
 *
 * <p>{@code Ephemeris.BUILTIN} selects the analytic ephemeris the SDK
 * carries, so this file runs anywhere.
 */
public final class Almanac {
    private Almanac() {}

    private static final int OFFSET_SECONDS = 20700;

    /**
     * A Julian day as the local clock reads it, which is what an almanac
     * prints; the offset is the one the request was made under.
     */
    private static String clock(double jd) {
        double local = (jd + OFFSET_SECONDS / 86400.0 + 0.5) % 1;
        long minutes = ((long) Math.rint(local * 1440)) % 1440;
        return String.format(Locale.ROOT, "%02d:%02d", minutes / 60, minutes % 60);
    }

    /**
     * The entity's name, or the key when the pack has none.
     *
     * <p>A locale pack names most of the catalogue and not all of it:
     * {@code masa} and {@code direction} have no entries in any of the five
     * the SDK ships, so an almanac falls back rather than refusing to print.
     */
    private static String name(Context ctx, String key) {
        try {
            return ctx.intl().entity(key).name();
        } catch (TeistroException e) {
            return key.substring(key.indexOf('.') + 1).toLowerCase(Locale.ROOT).replace('_', ' ');
        }
    }

    /**
     * One of the five limbs. The vara is one of them and is the day's own;
     * the other four are spans, and a day usually has two of each.
     */
    private static void limb(Context ctx, String label, List<? extends Span<? extends Catalogued>> spans) {
        List<String> printed = new ArrayList<>();
        for (Span<? extends Catalogued> span : spans) {
            // `whole` is the member's own span and `inside` the clipped
            // one, so a member that began yesterday says so rather than
            // looking as though it began at sunrise.
            String began = span.whole().fromJd() < span.inside().fromJd() ? "‹" : " ";
            String ends = span.whole().toJd() > span.inside().toJd() ? "›" : " ";
            printed.add(began + name(ctx, span.member().fullKey()) + " until " + clock(span.inside().toJd()) + ends);
        }
        System.out.printf(Locale.ROOT, "  %-10s %s%n", label, String.join("  ", printed));
    }

    /**
     * Runs the example.
     *
     * @param args unused
     */
    public static void main(String[] args) {
        Teistro teistro = Teistro.open();
        ContextOptions options = ContextOptions.builder()
                .profile("parashari-classical")
                .locale("ne-Deva-NP")
                .ephemeris(Ephemeris.BUILTIN)
                .build();
        try (Context ctx = teistro.context(options)) {
            Observer place = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));

            // ── One crossing for the whole week ───────────────────────────
            com.teispace.teistro.Almanac week = ctx.almanac().of(
                    date(Calendar.GREGORIAN, 2024, 6, 17),
                    date(Calendar.GREGORIAN, 2024, 6, 23),
                    place,
                    OFFSET_SECONDS);

            System.out.println(week.size() + " days at " + place.latitudeDeg().value() + "°N "
                    + place.longitudeDeg().value() + "°E, one crossing");
            System.out.println("calendar " + week.calendar().fullKey() + "   model " + week.model().split(",")[0]);
            System.out.println("");

            Day columns = week.decoded().day();
            for (AlmanacDay day : week) {
                int i = day.index();
                System.out.printf(Locale.ROOT, "%-12s %d-%02d-%02d   sunrise %s  sunset %s   %s %s%n",
                        name(ctx, day.day().vara().fullKey()),
                        columns.year(i), columns.month(i), columns.dayOfMonth(i),
                        clock(day.day().sunrise()), clock(day.day().sunset()),
                        name(ctx, day.month().amanta().fullKey()), name(ctx, day.month().paksha().fullKey()));

                // The five limbs.
                limb(ctx, "tithi", day.tithi());
                limb(ctx, "nakshatra", day.nakshatra());
                limb(ctx, "yoga", day.yoga());
                limb(ctx, "karana", day.karana());

                // The periods a day is planned around. Rahu kalam is the one
                // everybody checks; the choghadiya are what a shop opens on.
                List<String> kaalas = new ArrayList<>();
                for (KaalaPeriod k : day.kaalas()) {
                    kaalas.add(name(ctx, k.kaala().fullKey()) + " " + clock(k.at().fromJd()) + "–"
                            + clock(k.at().toJd()));
                }
                System.out.printf(Locale.ROOT, "  %-10s %s%n", "kaala", String.join("  ", kaalas));
                List<String> shown = day.choghadiya().stream()
                        .filter(ChoghadiyaPeriod::daytime)
                        .limit(3)
                        .map(c -> name(ctx, c.choghadiya().fullKey()) + " " + clock(c.at().fromJd()))
                        .toList();
                System.out.printf(Locale.ROOT, "  %-10s %s …%n", "choghadiya", String.join("  ", shown));
                Optional<Abhijit> abhijit = day.abhijit();
                if (abhijit.isPresent()) {
                    String effective = abhijit.get().effective() ? "" : "  (not effective on a Wednesday)";
                    System.out.printf(Locale.ROOT, "  %-10s %s–%s%s%n", "abhijit",
                            clock(abhijit.get().at().fromJd()), clock(abhijit.get().at().toJd()), effective);
                }
                // Absent is absent: no sankranti is an empty Optional, and a
                // Moon that did not rise inside the window contributes no
                // event at all.
                Optional<Double> sankranti = day.sankranti();
                if (sankranti.isPresent()) {
                    System.out.printf(Locale.ROOT, "  %-10s the Sun enters a new sign at %s%n", "sankranti",
                            clock(sankranti.get()));
                }
                List<String> moon = new ArrayList<>();
                for (MoonEventMoment e : day.moonEvents()) {
                    moon.add((e.rise() ? "RISE" : "SET") + " " + clock(e.instant()));
                }
                System.out.printf(Locale.ROOT, "  %-10s %s%n", "moon",
                        moon.isEmpty() ? "(neither rise nor set inside the window)" : String.join("  ", moon));
                System.out.println("");
            }

            // ── What the ragged layout costs a reader, which is nothing ───
            // Each day's lists are slices of one concatenated column, found
            // by adding up every earlier day's count. The layer does that sum
            // once when the batch is built, so `day.karana()` is a slice and
            // not a search.
            int counted = 0;
            for (AlmanacDay day : week) {
                counted += day.karana().size();
            }
            System.out.println(counted + " karanas across " + week.size() + " days, from one crossing");
            System.out.println("settings hash  " + ctx.settingsHash().substring(0, 16) + "…");

            // A day on its own is the range of one unwrapped: same crossing,
            // and the answer is a day rather than a list of one.
            AlmanacDay one = ctx.almanac().day(date(Calendar.GREGORIAN, 2024, 6, 21), place, OFFSET_SECONDS);
            System.out.println("day(one)       " + name(ctx, one.day().vara().fullKey()) + "  "
                    + one.horas().size() + " horas, " + one.muhurtas().size() + " muhurtas, "
                    + one.choghadiya().size() + " choghadiya");
        }
    }
}
