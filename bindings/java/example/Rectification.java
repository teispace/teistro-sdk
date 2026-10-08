import static com.teispace.teistro.Civil.at;
import static com.teispace.teistro.Civil.date;
import static com.teispace.teistro.Civil.ianaZone;

import java.util.Locale;

import com.teispace.teistro.Altitude;
import com.teispace.teistro.Calendar;
import com.teispace.teistro.CalendarDate;
import com.teispace.teistro.Chart;
import com.teispace.teistro.ChartBatch;
import com.teispace.teistro.ChartOptions;
import com.teispace.teistro.Context;
import com.teispace.teistro.ContextOptions;
import com.teispace.teistro.Ephemeris;
import com.teispace.teistro.Graha;
import com.teispace.teistro.Latitude;
import com.teispace.teistro.Longitude;
import com.teispace.teistro.Observer;
import com.teispace.teistro.PlacedGraha;
import com.teispace.teistro.Rashi;
import com.teispace.teistro.Teistro;
import com.teispace.teistro.ZoneResolution;

/**
 * Rectification: a birth time known only to the hour, narrowed by lagna.
 *
 * <p>A birth record that says "some time before dawn" is the commonest hard
 * case in the field. What narrows it is the <b>lagna</b>: it moves through
 * all twelve signs in a day, so it changes sign every couple of hours, and a
 * family that remembers the ascendant remembers something the clock does
 * not.
 *
 * <p>The point of this example is the shape of the call. A rectification
 * pass wants many charts at one place, and {@code foundMany} founds them in
 * <b>one crossing</b>: the settings are resolved once, the solar model is
 * built once, and the day each instant belongs to is reckoned against the
 * same sunrise. Founding them one at a time would give the same numbers and
 * pay the setup for every one of them.
 *
 * <p>{@code Ephemeris.BUILTIN} selects the analytic ephemeris the SDK
 * carries, so this file runs anywhere.
 */
public final class Rectification {
    private static final int FROM_HOUR = 0;
    private static final int TO_HOUR = 3;
    private static final int EVERY_MINUTES = 10;

    private Rectification() {}

    /**
     * Runs the example.
     *
     * @param args unused
     */
    public static void main(String[] args) {
        Teistro teistro = Teistro.open();
        // `nepali-default` is what a Nepali birth record is cast under, and
        // its frame is **topocentric**: the chart is seen from the hill the
        // record was written on rather than from the centre of the Earth. The
        // completion does that step itself over any provider
        // (`03-design/topocentric-measured.md`), so the analytic one below is
        // enough, and the steps printed at the end name it.
        ContextOptions options = ContextOptions.builder()
                .profile("nepali-default")
                .locale("ne-Deva-NP")
                .ephemeris(Ephemeris.BUILTIN)
                .build();
        try (Context ctx = teistro.context(options)) {
            // The record: a Bikram Sambat date, a place, and an hour nobody
            // is sure of. Everything below narrows the last of those.
            CalendarDate born = date(Calendar.BIKRAM_SAMBAT, 2045, 9, 17);
            Observer place = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));

            // One resolution fixes the zone and the offset; the candidates
            // are then arithmetic on the instant, which is what a Julian day
            // is for.
            ZoneResolution start = ctx.time().resolve(at(born, FROM_HOUR, 0), ianaZone("Asia/Kathmandu"));
            double step = EVERY_MINUTES / (24.0 * 60);
            int count = (TO_HOUR - FROM_HOUR) * 60 / EVERY_MINUTES;
            double[] instants = new double[count];
            for (int i = 0; i < count; i += 1) {
                instants[i] = start.instantJdUtc() + i * step;
            }

            // ── One crossing for every candidate ──────────────────────────
            ChartBatch charts = ctx.chart().foundMany(instants, place, start.offsetSeconds(), ChartOptions.none());

            System.out.println(charts.size() + " candidate charts, " + EVERY_MINUTES + " minutes apart, "
                    + "in one crossing");
            System.out.println("place  " + place.latitudeDeg().value() + "°N " + place.longitudeDeg().value()
                    + "°E   " + charts.kind().fullKey());
            System.out.println();
            System.out.println("local   lagna        sign            moon         bhava");
            System.out.println("─".repeat(58));

            Integer previous = null;
            for (Chart chart : charts) {
                int sign = (int) Math.floor(chart.lagnaDeg() / 30);
                PlacedGraha moon = chart.grahas().stream()
                        .filter(g -> g.graha() == Graha.MOON)
                        .findFirst()
                        .orElseThrow();
                String rashi = ctx.intl().entity(Rashi.of(sign).fullKey()).name();
                int minutes = FROM_HOUR * 60 + chart.index() * EVERY_MINUTES;
                String changed = previous != null && previous != sign ? "   ← lagna changes sign" : "";
                System.out.printf(Locale.ROOT, "%02d:%02d   %9.4f°  %-14s %9.4f°  %2d%s%n",
                        minutes / 60, minutes % 60, chart.lagnaDeg(), rashi, moon.longitudeDeg(),
                        moon.house().bhava(), changed);
                previous = sign;
            }

            // ── What the batch shares, and what it does not ───────────────
            // The place, the settings, the solar model and the completion
            // steps are one to a batch: they are what "the same chart at a
            // different minute" holds constant. The instant, the lagna, the
            // day and the timing are per chart. The provenance envelope
            // stamps the batch as a whole, so a rectification run reproduces
            // as one thing.
            System.out.println();
            System.out.println("model          " + charts.model());
            System.out.println("steps applied  " + String.join(", ", charts.stepsApplied()));
            System.out.println("settings hash  " + ctx.settingsHash().substring(0, 16) + "…");

            // A batch of one is the ordinary case, and `found` is the same
            // crossing with the batch unwrapped: the answer is a chart, not a
            // list of one.
            Chart single = ctx.chart().found(start.instantJdUtc(), place, start.offsetSeconds(), ChartOptions.none());
            System.out.println();
            System.out.printf(Locale.ROOT, "found(one)     lagna %.4f°  vara %s  ishtakaal %d:%d:%d  hora lord %s%n",
                    single.lagnaDeg(), single.day().vara().fullKey(),
                    single.timing().ghati(), single.timing().pala(), single.timing().vipala(),
                    single.timing().horaLord().fullKey());
            // The same crossing, so the same answer to the bit: `==` on two
            // doubles is a comparison of their bits for every value but NaN
            // and zero's sign.
            boolean agrees = single.lagnaDeg() == charts.get(0).lagnaDeg();
            System.out.printf(Locale.ROOT, "               and it agrees with the batch of one bit for bit: %b%n", agrees);
        }
    }
}
