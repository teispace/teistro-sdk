import static com.teispace.teistro.Civil.date;

import java.util.List;
import java.util.Locale;

import com.teispace.teistro.AlmanacDay;
import com.teispace.teistro.Altitude;
import com.teispace.teistro.Calendar;
import com.teispace.teistro.Context;
import com.teispace.teistro.ContextOptions;
import com.teispace.teistro.Ephemeris;
import com.teispace.teistro.Latitude;
import com.teispace.teistro.Longitude;
import com.teispace.teistro.Observer;
import com.teispace.teistro.Teistro;
import com.teispace.teistro.blob.Day;

/**
 * Nepal's official days, and the modern reckoning beside them.
 *
 * <p>Nepal's patro is computed by the Nepali committee from the Surya
 * Siddhanta: the tithi, nakshatra and yoga by the text with the committee's
 * bija, and the sunrise from a modern ephemeris. That is what
 * {@code nepalOfficial} opens. The modern reckoning, {@code nepali-drik},
 * reads the same day from the modern sky, and the two name a different
 * sunrise tithi on a few days a month.
 *
 * <p>What the shape teaches, and what a reader should copy:
 *
 * <ul>
 *   <li>The official day is a <b>preset</b>, not a recipe:
 *       {@code ContextOptions.builder().nepalOfficial(engine)}.
 *   <li>The Surya Siddhanta takes the engine <b>after</b> it in the chain for
 *       its sunrise; an app passes its own, {@code nepalOfficial(teimeris)}.
 *       This file passes none, so the sunrise is the built-in ephemeris's
 *       and it runs anywhere.
 *   <li>Where the two disagree, the reason is a tithi's end against the
 *       sunrise: the tithi the modern sky still has at sunrise had already
 *       ended by the text's, or the other way round. Each row prints both
 *       ends.
 * </ul>
 */
public final class NepalOfficial {
    private NepalOfficial() {}

    private static final int OFFSET_SECONDS = 20700;

    /** A Julian day as the local clock reads it. */
    private static String clock(double jd) {
        double local = (jd + OFFSET_SECONDS / 86400.0 + 0.5) % 1;
        long minutes = ((long) Math.rint(local * 1440)) % 1440;
        return String.format(Locale.ROOT, "%02d:%02d", minutes / 60, minutes % 60);
    }

    /** The same month under one context, in one crossing. */
    private static com.teispace.teistro.Almanac month(Context ctx, Observer place) {
        return ctx.almanac().of(
                date(Calendar.GREGORIAN, 2025, 11, 1),
                date(Calendar.GREGORIAN, 2025, 11, 30),
                place,
                OFFSET_SECONDS);
    }

    /**
     * Runs the example.
     *
     * @param args unused
     */
    public static void main(String[] args) {
        Teistro teistro = Teistro.open();
        ContextOptions officialOptions = ContextOptions.builder().nepalOfficial().locale("en-Latn").build();
        ContextOptions modernOptions = ContextOptions.builder()
                .profile("nepali-drik")
                .locale("en-Latn")
                .ephemeris(Ephemeris.BUILTIN)
                .build();
        try (Context official = teistro.context(officialOptions); Context modern = teistro.context(modernOptions)) {
            Observer place = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
            com.teispace.teistro.Almanac days = month(official, place);
            List<AlmanacDay> ours = days.stream().toList();
            List<AlmanacDay> theirs = month(modern, place).stream().toList();

            System.out.println("official " + official.profile() + ", modern " + modern.profile());
            Day columns = days.decoded().day();
            int differ = 0;
            for (int i = 0; i < ours.size(); i++) {
                // The first span of a day's limb is the one standing at its
                // sunrise.
                var mine = ours.get(i).tithi().get(0);
                var other = theirs.get(i).tithi().get(0);
                if (mine.member() == other.member()) {
                    continue;
                }
                differ++;
                System.out.printf(Locale.ROOT, "%d-%02d-%02d  sunrise %s  official %s until %s  modern %s until %s%n",
                        columns.year(i), columns.month(i), columns.dayOfMonth(i),
                        clock(ours.get(i).day().sunrise()),
                        official.intl().entity(mine.member().fullKey()).name(), clock(mine.whole().toJd()),
                        official.intl().entity(other.member().fullKey()).name(), clock(other.whole().toJd()));
            }
            System.out.println(differ + " of " + ours.size() + " days name another sunrise tithi");
        }
    }
}
