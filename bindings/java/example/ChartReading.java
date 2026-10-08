import static com.teispace.teistro.Civil.at;
import static com.teispace.teistro.Civil.date;
import static com.teispace.teistro.Civil.ianaZone;

import java.util.List;
import java.util.Locale;
import java.util.Optional;

import com.teispace.teistro.Altitude;
import com.teispace.teistro.Calendar;
import com.teispace.teistro.CalendarDate;
import com.teispace.teistro.Catalogued;
import com.teispace.teistro.Chart;
import com.teispace.teistro.ChartLayout;
import com.teispace.teistro.ChartOptions;
import com.teispace.teistro.Context;
import com.teispace.teistro.ContextOptions;
import com.teispace.teistro.DerivedPoint;
import com.teispace.teistro.Drawing;
import com.teispace.teistro.DrawnCell;
import com.teispace.teistro.Drishti;
import com.teispace.teistro.Ephemeris;
import com.teispace.teistro.Graha;
import com.teispace.teistro.GrahaState;
import com.teispace.teistro.Latitude;
import com.teispace.teistro.Longitude;
import com.teispace.teistro.Observer;
import com.teispace.teistro.Point;
import com.teispace.teistro.Rashi;
import com.teispace.teistro.ServiceBhava;
import com.teispace.teistro.Teistro;
import com.teispace.teistro.Varga;
import com.teispace.teistro.VargaChart;
import com.teispace.teistro.VargaPlacement;
import com.teispace.teistro.ZoneResolution;

/**
 * A chart reading: one call for the whole document a reader interprets.
 *
 * <p>{@code BirthChart.java} placed the grahas. A reading is what comes
 * after: the divisional charts, the houses, what each graha <b>is</b> rather
 * than where it is, which grahas look at which, and the derived points. Each
 * is a section a request asks for by name, and each is computed from the
 * same founded chart in the same crossing, so asking for all of them costs
 * one call, and asking for none of them costs nothing
 * ({@code docs/03-design/chart-reading.md}).
 *
 * <p>What it teaches:
 *
 * <ol>
 *   <li><b>Sections are asked for.</b> {@code vargas}, {@code aspects},
 *       {@code points}, {@code houses} and {@code state} are off by default,
 *       so a birth chart does not pay for twenty-one divisional charts it
 *       will not show.
 *   <li><b>Vargottama is a comparison, not a flag</b>: a graha whose navamsha
 *       sign is the sign it stands in. The layer gives both signs.
 *   <li><b>A dignity and a house are different questions</b>, answered by
 *       different sections: {@code states()} says how a graha fares in its
 *       sign, {@code bhavas()} who rules a house.
 *   <li><b>The drishti are ragged</b>: how many there are depends on where
 *       the grahas stand, not on how many grahas there are.
 *   <li><b>A drawing is geometry, not pixels</b>: each cell's outline in a
 *       unit square, the sign and house it shows and the grahas in it, so any
 *       renderer draws the same chart.
 * </ol>
 *
 * <p>The record is {@code BirthChart.java}'s own, so the two can be read
 * side by side.
 */
public final class ChartReading {
    private ChartReading() {}

    /** A catalogue member's name in the context's locale. */
    private static String name(Context ctx, Catalogued member) {
        return ctx.intl().entity(member.fullKey()).name();
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
                .locale("ne-Deva-NP")
                .ephemeris(Ephemeris.BUILTIN)
                .build();
        try (Context ctx = teistro.context(options)) {
            CalendarDate born = date(Calendar.BIKRAM_SAMBAT, 2042, 9, 17);
            ZoneResolution when = ctx.time().resolve(at(born, 0, 20), ianaZone("Asia/Kathmandu"));

            // ── One call for every section ─────────────────────────────────
            Chart chart = ctx.chart().found(
                    when.instantJdUtc(),
                    new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400)),
                    when.offsetSeconds(),
                    ChartOptions.builder()
                            .vargas(Varga.D9, Varga.D10)
                            .aspects(true)
                            .points(true)
                            .houses(true)
                            .state(true)
                            .drawing(ChartLayout.NORTH_INDIAN, Varga.D9)
                            .build());
            VargaChart navamsha = chart.vargas().get(0);
            VargaChart dasamsha = chart.vargas().get(1);

            System.out.println("reading  BS 2042-09-17  00:20  Kathmandu");
            Rashi lagna = Rashi.of((int) Math.floor(chart.lagnaDeg() / 30));
            System.out.printf(Locale.ROOT, "lagna    %s %.4f°   D9 %s   D10 %s%n",
                    name(ctx, lagna), chart.lagnaDeg() % 30,
                    name(ctx, navamsha.lagna().sign()), name(ctx, dasamsha.lagna().sign()));

            // ── What each graha is ─────────────────────────────────────────
            System.out.println();
            System.out.println("graha        house  dignity          age            D9 sign      vargottama  combust");
            System.out.println("─".repeat(88));
            List<GrahaState> states = chart.states();
            for (int j = 0; j < states.size(); j += 1) {
                GrahaState state = states.get(j);
                VargaPlacement inNavamsha = navamsha.grahas().get(j).at();
                String vargottama = inNavamsha.sign() == inNavamsha.rashi() ? "yes" : "no";
                System.out.printf(Locale.ROOT, "%-12s %5d  %-16s %-14s %-12s %-11s %s%n",
                        name(ctx, state.graha()), state.house(),
                        name(ctx, state.dignity()), name(ctx, state.age()),
                        name(ctx, inNavamsha.sign()), vargottama,
                        state.combustion().burning().key());
            }

            // ── The houses ─────────────────────────────────────────────────
            // The tenth house, by the houses service: the sign its middle falls
            // in, that sign's lord, and which kind of house it is.
            ServiceBhava tenth = chart.bhavas().get(9);
            System.out.println();
            System.out.println("bhava 10 " + name(ctx, tenth.sign()) + ", ruled by " + name(ctx, tenth.lord())
                    + " (" + tenth.quadrant().key() + ")");

            // ── Which grahas look at the Moon ──────────────────────────────
            // `houses` counts inclusively from the looking graha's sign, so the
            // seventh is the house opposite it.
            System.out.println("drishti  " + chart.aspects().size() + " under "
                    + chart.batch().decoded().drishtiTable() + "; on the Moon:");
            for (Drishti drishti : chart.aspects()) {
                if (drishti.to() == Graha.MOON) {
                    System.out.printf(Locale.ROOT, "         %-12s house %2d from it  %s%n",
                            name(ctx, drishti.fromGraha()), drishti.houses(), drishti.strength().key());
                }
            }

            // ── The derived points ─────────────────────────────────────────
            // Gulika is Saturn's portion of the day's arc, which is why a chart
            // with no day to divide has none -- the section is ragged for that.
            Optional<DerivedPoint> gulika = chart.points().stream()
                    .filter(found -> found.point() == Point.GULIKA)
                    .findFirst();
            String where = gulika
                    .map(found -> String.format(Locale.ROOT, "%s %.4f°",
                            name(ctx, found.sign()), found.longitudeDeg() % 30))
                    .orElse("none");
            System.out.println("gulika   " + where + "   " + chart.points().size() + " points");

            // ── The navamsha, drawn ────────────────────────────────────────
            // A North Indian chart keeps its houses still and moves the signs,
            // so the lagna is always the top diamond; the cell says which sign
            // landed there.
            Drawing drawn = chart.drawings().get(0);
            DrawnCell risen = drawn.cells().stream().filter(DrawnCell::lagna).findFirst().orElseThrow();
            String layout = drawn.layoutKey().substring(drawn.layoutKey().lastIndexOf('.') + 1);
            System.out.println("drawing  " + layout.toLowerCase(Locale.ROOT) + " "
                    + drawn.varga().key().toLowerCase(Locale.ROOT) + ": " + drawn.cells().size() + " cells, "
                    + "lagna in house " + risen.house() + " (" + name(ctx, risen.sign()) + "), grahas there: "
                    + risen.bodies().size());
            System.out.println("settings hash  " + ctx.settingsHash().substring(0, 16) + "…");
        }
    }
}
