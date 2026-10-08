import static com.teispace.teistro.Civil.at;
import static com.teispace.teistro.Civil.date;
import static com.teispace.teistro.Civil.ianaZone;

import java.util.Arrays;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.stream.Collectors;
import java.util.stream.IntStream;

import com.teispace.teistro.Altitude;
import com.teispace.teistro.AnnualDasha;
import com.teispace.teistro.Calendar;
import com.teispace.teistro.CalendarDate;
import com.teispace.teistro.Chart;
import com.teispace.teistro.ChartOptions;
import com.teispace.teistro.Context;
import com.teispace.teistro.ContextOptions;
import com.teispace.teistro.DashaPeriod;
import com.teispace.teistro.DashaSystem;
import com.teispace.teistro.Ephemeris;
import com.teispace.teistro.HarshaBala;
import com.teispace.teistro.Latitude;
import com.teispace.teistro.Longitude;
import com.teispace.teistro.Member;
import com.teispace.teistro.Observer;
import com.teispace.teistro.OfficeBearers;
import com.teispace.teistro.Pravesha;
import com.teispace.teistro.Saham;
import com.teispace.teistro.TajikaMatter;
import com.teispace.teistro.TajikaSaham;
import com.teispace.teistro.Teistro;
import com.teispace.teistro.TeistroException;
import com.teispace.teistro.YearClaim;
import com.teispace.teistro.YearLord;
import com.teispace.teistro.ZoneResolution;

/**
 * The annual chart: the one instant every Tajika judgement is made from.
 *
 * <p>A birth chart is cast for a birth. An <b>annual</b> chart is cast for
 * the moment the Sun comes back to the longitude it held then: once a year,
 * about twenty minutes earlier than the clock would say, and never on the
 * birthday itself ({@code docs/03-design/annual-chart.md}).
 *
 * <p>What it teaches:
 *
 * <ol>
 *   <li><b>The instant, then the chart.</b> Whether the annual chart is cast
 *       for the birthplace or for where you live now is a question the
 *       schools answer differently, so the SDK answers the instant and casts
 *       the year's chart only where you name a place ({@code "birth"} or a
 *       residence) in the same {@code varsha} request.
 *   <li><b>Which longitude is a choice with a name.</b> {@code SIDEREAL} is
 *       the tradition's; {@code TROPICAL} is the Western solar return and is
 *       most of a circle of lagna away by the fortieth year; {@code MEAN} is
 *       the older arithmetic and needs no ephemeris at all. None of them is a
 *       fallback for another.
 *   <li><b>Fewer than you asked for is the answer</b>, not a refusal: an
 *       ephemeris that ends before your hundredth year says so by giving you
 *       the years it has.
 * </ol>
 *
 * <p>The record is {@code BirthChart.java}'s own, so the two can be read
 * side by side.
 */
public final class AnnualChart {
    private AnnualChart() {}

    /** A chart founded at the birth, with the varsha request given. */
    private static Chart found(Context ctx, ZoneResolution when, Observer place, Object varsha) {
        return ctx.chart().found(when.instantJdUtc(), place, when.offsetSeconds(),
                ChartOptions.builder().varsha(varsha).build());
    }

    /** The thirtieth year's own chart, founded with the varsha request given. */
    private static com.teispace.teistro.AnnualChart thirtieth(
            Context ctx, ZoneResolution when, Observer place, Object varsha) {
        com.teispace.teistro.AnnualChart annual = found(ctx, when, place, varsha).praveshas().get(29).annual();
        if (annual == null) {
            throw new IllegalStateException("a varsha request naming a place casts the year's chart");
        }
        return annual;
    }

    /** Members' keys, joined. */
    private static String keys(List<? extends Member> members, String separator) {
        return members.stream().map(Member::key).collect(Collectors.joining(separator));
    }

    /**
     * Runs the example.
     *
     * @param args unused
     */
    public static void main(String[] args) {
        Teistro teistro = Teistro.open();
        ContextOptions options = ContextOptions.builder()
                .profile("nepali-default")
                .ephemeris(Ephemeris.BUILTIN)
                .build();
        try (Context ctx = teistro.context(options)) {
            CalendarDate birthDay = date(Calendar.GREGORIAN, 1990, 4, 14);
            ZoneResolution when = ctx.time().resolve(at(birthDay, 5, 30), ianaZone("Asia/Kathmandu"));
            Observer place = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));

            // ── The years a birth opens ──────────────────────────────────
            Chart chart = found(ctx, when, place, Map.of("reading", "SIDEREAL", "through", 40));
            List<Pravesha> years = chart.praveshas();
            System.out.println("returns computed: " + years.size());
            Pravesha thirtieth = years.stream().filter(one -> one.year() == 30).findFirst().orElseThrow();
            System.out.printf(Locale.ROOT, "the thirtieth year opens at jd %.6f%n", thirtieth.instant());

            // A return is about a sidereal year after the last, never a
            // calendar one.
            double[] gaps = IntStream.range(1, years.size())
                    .mapToDouble(i -> years.get(i).instant() - years.get(i - 1).instant())
                    .toArray();
            System.out.printf(Locale.ROOT, "between returns: %.4f to %.4f days%n",
                    Arrays.stream(gaps).min().orElseThrow(), Arrays.stream(gaps).max().orElseThrow());

            // ── The chart of that year, cast where you choose ────────────
            Chart annual = ctx.chart().found(thirtieth.instant(), place, when.offsetSeconds(), ChartOptions.none());
            System.out.printf(Locale.ROOT, "natal lagna %.3f°, annual lagna %.3f°%n",
                    chart.lagnaDeg(), annual.lagnaDeg());

            // ── Its five office-bearers, cast where you say ───────────────
            // Here the birthplace, the one Tajika text read casts every chart
            // for; a residence is {"observer": {...}, ...} instead.
            Pravesha cast = found(ctx, when, place, Map.of("through", 30, "place", "birth")).praveshas().get(29);
            com.teispace.teistro.AnnualChart year = cast.annual();
            if (year == null) {
                throw new IllegalStateException("a varsha request naming a place casts the year's chart");
            }
            OfficeBearers b = year.officeBearers();
            String five = keys(List.of(b.muntha(), b.janmaLagna(), b.varshaLagna(), b.triRashi(), b.dinaRatri()), " ");
            String part = year.byDay() ? "by day" : "by night";
            System.out.println("muntha in " + cast.muntha().sign().key() + "; office-bearers " + five + ", " + part);

            // ── And the lord of that year, with the reason ─────────────────
            YearLord lord = year.yearLord();
            System.out.println("year lord " + lord.graha().key() + " at " + lord.vishwa()
                    + ", chosen " + lord.chosen().key());
            for (YearClaim claim : lord.claims()) {
                String aspects = claim.aspectsLagna() ? "aspects" : "does not aspect";
                System.out.printf(Locale.ROOT, "  %-8s %s  %d portfolio(s)  %s the lagna%n",
                        claim.graha().key(), claim.vishwa(), claim.portfolios(), aspects);
            }

            // ── The sixteen Tajika yogas answer a matter, not a chart ────
            // Fourteen of them judge the lagnesha against the lord of the house
            // you ask about, so you name the houses: marriage (7) and career
            // (10) here.
            com.teispace.teistro.AnnualChart judged = thirtieth(ctx, when, place,
                    Map.of("through", 30, "place", "birth", "matters", List.of(7, 10)));
            for (TajikaMatter matter : judged.matters()) {
                String held = matter.held().stream().map(one -> one.yoga().key()).collect(Collectors.joining(", "));
                System.out.println("house " + matter.house() + ": " + matter.lagnesha().key() + " with "
                        + matter.karyesha().key() + ", held " + (held.isEmpty() ? "none" : held)
                        + "; not answered " + keys(matter.unanswered(), ", "));
            }

            // ── The sahams: forty-one sensitive points, each a − b + c ───
            // Name the ones you want, or "all"; each comes back with its sign,
            // that sign's lord and the house it fell in, as the source reads
            // them.
            com.teispace.teistro.AnnualChart points = thirtieth(ctx, when, place,
                    Map.of("through", 30, "place", "birth",
                            "sahams", List.of(Saham.PUNYA, Saham.VIVAHA, Saham.KARYA_SIDDHI)));
            for (TajikaSaham point : points.sahams()) {
                String added = point.addedSign() ? " (a sign added)" : "";
                System.out.printf(Locale.ROOT, "%-12s %6.2f°  %s, lord %s, house %d%s%n",
                        point.saham().key(), point.longitudeDeg(), point.sign().key(), point.lord().key(),
                        point.house(), added);
                // Its strength is the source's clauses, reported and never scored.
                String strong = keys(point.strong(), ", ");
                String weak = keys(point.weak(), ", ");
                System.out.println("  strong: " + (strong.isEmpty() ? "none" : strong)
                        + "; weak: " + (weak.isEmpty() ? "none" : weak));
            }
            // And the seven's Harsha bala that year: four places each is happy in.
            System.out.println(points.harsha().stream()
                    .map((HarshaBala h) -> h.graha().key() + " " + h.total())
                    .collect(Collectors.joining(", ")));

            // ── The annual dashas: the year divided among its lords ───────
            // The Mudda runs round the nine from the birth nakshatra's lord,
            // one lord further each year; the Patyayini is read from the year's
            // own chart, and its lagna's share is a sign's. The Sun is read
            // over the year once for both, and each year closes on the next
            // return.
            com.teispace.teistro.AnnualChart divided = thirtieth(ctx, when, place,
                    Map.of("through", 30, "place", "birth",
                            "dashas", List.of(DashaSystem.MUDDA, DashaSystem.PATYAYINI)));
            for (AnnualDasha dasha : divided.dashas()) {
                String days = dasha.periods().stream()
                        .filter(p -> p.level() == 1)
                        .map((DashaPeriod p) -> String.format(Locale.ROOT, "%s %.1f",
                                p.sign() != null ? p.sign().key() : p.lord().key(),
                                p.span().toJd() - p.span().fromJd()))
                        .collect(Collectors.joining(", "));
                System.out.println(dasha.system().key() + ": " + days);
            }

            // ── The readings are named, and they are not each other ──────
            for (String reading : List.of("SIDEREAL", "TROPICAL", "MEAN")) {
                List<Pravesha> one = found(ctx, when, place, Map.of("reading", reading, "through", 30)).praveshas();
                double apart = (one.get(29).instant() - years.get(29).instant()) * 24;
                System.out.printf(Locale.ROOT, "%-9s thirtieth year, %.2f hours from the sidereal one%n",
                        reading, apart);
            }

            // ── What it refuses, and by which field ──────────────────────
            try {
                found(ctx, when, place, Map.of("reading", "SIDEREAL", "through", 0));
            } catch (TeistroException error) {
                System.out.println("refused  " + error.field() + ": " + error.getMessage());
            }
        }
    }
}
