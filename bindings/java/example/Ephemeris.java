import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.function.IntToDoubleFunction;
import java.util.stream.Collectors;

import com.teispace.teistro.Ayanamsha;
import com.teispace.teistro.Body;
import com.teispace.teistro.Context;
import com.teispace.teistro.ContextOptions;
import com.teispace.teistro.Frame;
import com.teispace.teistro.Graha;
import com.teispace.teistro.PositionGrid;
import com.teispace.teistro.PositionRequest;
import com.teispace.teistro.Rashi;
import com.teispace.teistro.Teistro;
import com.teispace.teistro.TimeScale;
import com.teispace.teistro.blob.Positions;
import com.teispace.teistro.record.Provenance;
import com.teispace.teistro.record.ProviderStamp;

/**
 * A year of the sky in one call, and what to do with the columns.
 *
 * <p>The boundary takes a <b>grid</b> (instants by bodies) and answers with
 * a result blob whose sections are columns. That shape is the whole reason
 * a year of positions costs one crossing rather than 365, and it is what a
 * service computing tables, transits or ingresses should be using.
 *
 * <p>Three things this example is really about:
 *
 * <ol>
 *   <li><b>One call, not a loop.</b> 366 instants by 3 bodies is 1098 cells
 *       in a single request. Asking day by day would cross the boundary 366
 *       times and recompute the provider's own setup each time.
 *   <li><b>The columns are views, not copies.</b>
 *       {@code positions.decoded().cells().lon(row)} reads a double straight
 *       out of the blob's own bytes, so a table of a million cells costs one
 *       allocation rather than a million objects.
 *   <li><b>What the answer says about itself.</b> Every result carries the
 *       steps applied and a provenance envelope with the settings hash: the
 *       two things a cache key and an audit trail are made of.
 * </ol>
 *
 * <p>{@code Ephemeris.BUILTIN} computes with the analytic ephemeris the SDK
 * carries, so this file runs anywhere with nothing installed. It is the
 * fallback rather than the intended path (most consumers should be on a
 * real engine) but it is astronomy: the scans below find sign ingresses and
 * retrograde stations because the sky has them.
 */
public final class Ephemeris {
    private Ephemeris() {}

    /** A year from the start of 2025, one sample a day at noon UTC. */
    private static final double START_JD = 2460676.5;
    private static final int DAYS = 366;

    /**
     * A body is what an ephemeris answers; a graha is what a chart names.
     * They are different catalogues and the lunar node is where they part
     * ({@code MEAN_NODE} is the body, {@code RAHU} the graha) so the two are
     * paired explicitly rather than derived from each other's spelling.
     *
     * @param body what the ephemeris is asked for
     * @param graha what a chart calls it
     */
    private record Tracked(Body body, Graha graha) {}

    private static final List<Tracked> BODIES = List.of(
            new Tracked(Body.SUN, Graha.SUN),
            new Tracked(Body.MARS, Graha.MARS),
            new Tracked(Body.MEAN_NODE, Graha.RAHU));

    /**
     * A day on which a body entered a sign.
     *
     * @param day the day, from 0
     * @param sign the sign it entered
     */
    private record Ingress(int day, Rashi sign) {}

    /**
     * A day on which a body turned.
     *
     * @param day the day, from 0
     * @param into "retrograde" or "direct"
     */
    private record Turn(int day, String into) {}

    /**
     * Every day on which a body changed sign.
     *
     * <p>Reads one body's column out of the grid. Cells run instants
     * outermost, so body {@code column} at day {@code i} is cell
     * {@code i * stride + column}.
     */
    private static List<Ingress> ingresses(IntToDoubleFunction longitudes, int dayCount, int stride, int column) {
        List<Ingress> found = new ArrayList<>();
        Rashi previous = null;
        for (int day = 0; day < dayCount; day++) {
            Rashi sign = Rashi.of((int) Math.floor(longitudes.applyAsDouble(day * stride + column) / 30.0));
            if (previous != null && sign != previous) {
                found.add(new Ingress(day, sign));
            }
            previous = sign;
        }
        return found;
    }

    /**
     * Every day on which a body turned, direct to retrograde or back.
     *
     * <p>The same shape as {@code ingresses} over a different column: a
     * station is a sign change in {@code lonSpeed} rather than in
     * {@code lon}. One grid answers both, which is the reason to ask for a
     * grid.
     */
    private static List<Turn> stations(IntToDoubleFunction speeds, int dayCount, int stride, int column) {
        List<Turn> found = new ArrayList<>();
        for (int day = 1; day < dayCount; day++) {
            double before = speeds.applyAsDouble((day - 1) * stride + column);
            double after = speeds.applyAsDouble(day * stride + column);
            if ((before < 0) != (after < 0)) {
                found.add(new Turn(day, after < 0 ? "retrograde" : "direct"));
            }
        }
        return found;
    }

    /**
     * Runs the example.
     *
     * @param args unused
     */
    public static void main(String[] args) {
        Teistro teistro = Teistro.open();
        // This class is called `Ephemeris`, so the SDK's enum of the same
        // name is spelled in full.
        ContextOptions options = ContextOptions.builder()
                .profile("nepali-default")
                .locale("ne-Deva-NP")
                .ephemeris(com.teispace.teistro.Ephemeris.BUILTIN)
                .build();
        try (Context ctx = teistro.context(options)) {
            // The binding already refuses a library that is not the build it
            // was generated from; `teistro.buildInfo()` says what it did load
            // -- the SDK and ABI versions, the target and the commit -- which
            // is for a bug report. What computed an answer is stamped on the
            // answer, and is printed last.

            // ── One call for the whole year ───────────────────────────────
            Frame canonical = teistro.canonicalFrame();
            Frame frame = new Frame(Ayanamsha.LAHIRI, canonical.centre(), canonical.equinox(),
                    canonical.coordinates(), true, canonical.lightTime(), canonical.aberration(),
                    canonical.deflection(), canonical.nutation());
            double[] instants = new double[DAYS];
            for (int day = 0; day < DAYS; day++) {
                instants[day] = START_JD + day;
            }
            PositionGrid sky = ctx.positions(new PositionRequest(
                    TimeScale.UT1, teistro.packFrame(frame), true, null, instants,
                    BODIES.stream().map(Tracked::body).toList()));
            System.out.println("grid     " + sky.instantCount() + " instants x " + sky.bodyCount()
                    + " bodies = " + sky.cellCount() + " cells in one call");

            // ── The columns ───────────────────────────────────────────────
            // Point 2 above: each column is a getter over the blob's own
            // bytes, eight to a double, and nothing is copied out of it.
            Positions.Cells cells = sky.decoded().cells();
            System.out.println("columns  lon holds " + cells.length() + " doubles in "
                    + cells.length() * Double.BYTES + " bytes");

            // ── What the columns are for ──────────────────────────────────
            System.out.println();
            for (int column = 0; column < BODIES.size(); column++) {
                Graha graha = BODIES.get(column).graha();
                String name = ctx.intl().entity(graha.fullKey()).name();
                List<Ingress> crossings = ingresses(cells::lon, DAYS, sky.bodyCount(), column);
                List<Turn> turns = stations(cells::lonSpeed, DAYS, sky.bodyCount(), column);
                double speed = cells.lonSpeed(column);
                String direction = speed < 0 ? "retrograde" : "direct";
                System.out.printf(Locale.ROOT, "  %-10s %-8s %-10s at %+8.4f°/day, %d sign change(s), %d station(s)%n",
                        graha.key(), name, direction, speed, crossings.size(), turns.size());
                for (Ingress crossing : crossings.subList(0, Math.min(3, crossings.size()))) {
                    String signName = ctx.intl().entity(crossing.sign().fullKey()).name();
                    System.out.printf(Locale.ROOT, "      day %3d  enters %-12s %s%n",
                            crossing.day(), crossing.sign().key(), signName);
                }
                if (crossings.size() > 3) {
                    System.out.println("      … and " + (crossings.size() - 3) + " more");
                }
                for (Turn turn : turns) {
                    System.out.printf(Locale.ROOT, "      day %3d  turns  %s%n", turn.day(), turn.into());
                }
            }

            // ── What the answer says about itself ─────────────────────────
            System.out.println();
            String steps = sky.stepsApplied().stream()
                    .map(step -> step.name() + ":" + step.implementation().key())
                    .collect(Collectors.joining(", "));
            System.out.println("steps    " + steps);
            Provenance provenance = sky.provenance();
            System.out.println("profile  " + provenance.profile());
            System.out.println("hash     " + provenance.settingsHash());
            System.out.println("         two contexts with the same settings hash compute the"
                    + " same numbers, so it is the cache key");
            // The value's content hash is taken over its canonical JSON,
            // byte-identical in every binding, which is what makes a stored
            // result checkable.
            System.out.println("content  " + provenance.contentHash().substring(0, 16) + "…");
            ProviderStamp provider = provenance.provider();
            System.out.println("provider " + provider.name() + " " + provider.version()
                    + " (data " + provider.dataVersion() + ")");
        }
    }
}
