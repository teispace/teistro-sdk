import java.io.FileNotFoundException;
import java.util.List;
import java.util.Locale;
import java.util.Optional;
import java.util.stream.Collectors;

import com.teispace.teistro.Ayanamsha;
import com.teispace.teistro.Body;
import com.teispace.teistro.Context;
import com.teispace.teistro.ContextOptions;
import com.teispace.teistro.EphemerisProvider;
import com.teispace.teistro.Frame;
import com.teispace.teistro.PositionAnswer;
import com.teispace.teistro.PositionGrid;
import com.teispace.teistro.PositionQuery;
import com.teispace.teistro.PositionRequest;
import com.teispace.teistro.ProviderCode;
import com.teispace.teistro.ProviderException;
import com.teispace.teistro.Teistro;
import com.teispace.teistro.TeistroException;
import com.teispace.teistro.TimeScale;
import com.teispace.teistro.record.ProviderStamp;

/**
 * An ephemeris of your own, and what happens when it goes wrong.
 *
 * <p>The SDK computes no positions itself: it asks a <b>provider</b>, and a
 * provider written in Java is a first-class one. That is the point of the
 * port: an application that already has an ephemeris, a cache, or a table
 * of precomputed positions can put it behind the SDK and get the whole
 * chart layer for free.
 *
 * <p>The contract is small and worth reading carefully:
 *
 * <ul>
 *   <li><b>One call for the whole grid, never a loop.</b> The SDK hands over
 *       every instant and every body at once and expects every cell back.
 *   <li><b>Answer empty for "not in that frame".</b> A provider that
 *       computes equatorial positions says so once, in {@code nativeFrame},
 *       and answers empty when asked for anything else; the SDK then asks
 *       again in the provider's own frame and completes the rest itself,
 *       stamping each step. This is why an engine that knows nothing about
 *       the ecliptic can still serve a Vedic chart.
 *   <li><b>Say what you cover.</b> {@code bodies} is checked <i>before</i>
 *       the provider is called, so a body it does not answer is refused by
 *       name rather than by a wrong answer. An instant outside
 *       {@code jdMin}..{@code jdMax} is never asked for: its cells come back
 *       {@code OUT_OF_RANGE} and the rest are answered, as they would be
 *       from any engine.
 *   <li><b>Throwing is allowed.</b> An exception is carried across the
 *       boundary as a refusal code and thrown again on the caller's side,
 *       so the sentence is not lost. That matters more here than anywhere:
 *       an exception that escaped an FFM upcall would end the JVM, so the
 *       binding catches everything and keeps it for the caller.
 * </ul>
 */
public final class YourOwnEphemeris {
    private YourOwnEphemeris() {}

    /**
     * An ephemeris backed by whatever you already have.
     *
     * <p>This one is a two-body toy, a circular Sun and Moon, standing in
     * for the real thing: a {@code .se1} reader, a JPL kernel, a database of
     * precomputed rows, or a cache in front of any of them. What matters is
     * the shape, not the arithmetic.
     */
    static class TableEphemeris extends EphemerisProvider {
        int calls;
        int cells;
        int refusals;
        /**
         * The one frame this provider computes in, as the port packs it.
         * Null means "answer whatever is asked", which is what a provider
         * may say only when it really can.
         */
        private final Long wantedFrame;

        TableEphemeris() {
            this.wantedFrame = null;
        }

        TableEphemeris(Teistro teistro) {
            this.wantedFrame = teistro.packFrame(teistro.canonicalFrame());
        }

        @Override
        public String name() {
            return "table-ephemeris";
        }

        @Override
        public String version() {
            return "1.0.0";
        }

        // What identifies the data, not the code. A result's provenance
        // carries it, so two runs against different data are told apart
        // even when the code is identical.
        @Override
        public String dataVersion() {
            return "demo-rows-2025a";
        }

        @Override
        public List<Body> bodies() {
            return List.of(Body.SUN, Body.MOON);
        }

        // Cover only what you have. A request outside this is refused
        // before `positions` is ever called.
        @Override
        public double jdMin() {
            return 2451545.0;
        }

        @Override
        public double jdMax() {
            return 2469807.0;
        }

        @Override
        public Optional<PositionAnswer> positions(PositionQuery query) throws Exception {
            // **Check the frame first.** Answering at all asserts that the
            // answer is in the frame that was asked for; a provider that
            // computes only its own must say so by answering empty, and
            // the SDK then asks again in `nativeFrame` and completes the
            // rest. This one computes tropical ecliptic longitudes and
            // nothing else.
            if (wantedFrame != null && query.frameBits() != wantedFrame) {
                refusals += 1;
                return Optional.empty();
            }

            calls += 1;
            cells += query.cellCount();

            // Cells run instants outermost: cell `i * bodies + j` is
            // instant `i`, body `j`. Building the columns in that order is
            // the whole of the contract.
            int count = query.cellCount();
            double[] lon = new double[count];
            double[] speed = new double[count];
            int cell = 0;
            for (double jd : query.jds()) {
                double days = jd - 2451545.0;
                for (Body body : query.bodies()) {
                    double rate = body == Body.SUN ? 0.9856 : 13.1764;
                    double start = body == Body.SUN ? 280.46 : 218.32;
                    lon[cell] = ((start + rate * days) % 360.0 + 360.0) % 360.0;
                    speed[cell] = rate;
                    cell += 1;
                }
            }
            double[] lat = new double[count];
            double[] dist = new double[count];
            java.util.Arrays.fill(dist, 1.0);
            // Speeds are optional; a column left out is zeroes. Answering
            // `false` from `speeds()` would tell the SDK not to expect them.
            return Optional.of(PositionAnswer.of(lon, lat, dist).withLonSpeed(speed));
        }
    }

    /** A provider that fails the way a real one does: with a sentence. */
    static final class Broken extends TableEphemeris {
        @Override
        public String name() {
            return "broken";
        }

        @Override
        public Optional<PositionAnswer> positions(PositionQuery query) throws Exception {
            throw new FileNotFoundException("a data file is missing: de431.eph is not where the index says");
        }
    }

    private static ContextOptions with(EphemerisProvider provider) {
        return ContextOptions.builder().profile("parashari-classical").provider(provider).build();
    }

    /**
     * Runs the example.
     *
     * @param args unused
     */
    public static void main(String[] args) {
        Teistro teistro = Teistro.open();

        // ── The happy path ────────────────────────────────────────────────
        TableEphemeris provider = new TableEphemeris();
        try (Context ctx = teistro.context(with(provider))) {
            double[] week = new double[7];
            for (int day = 0; day < 7; day++) {
                week[day] = 2451545.0 + day;
            }
            PositionGrid sky = ctx.positions(week, List.of(Body.SUN, Body.MOON));
            System.out.println("asked    " + provider.calls + " time(s) for " + provider.cells + " cells");
            System.out.println("answered " + sky.cellCount() + " cells over " + sky.instantCount() + " days");
            System.out.printf(Locale.ROOT, "  sun  %8.4f° -> %8.4f° in a week%n",
                    sky.at(0, 0).longitude(), sky.at(6, 0).longitude());
            System.out.printf(Locale.ROOT, "  moon %8.4f° -> %8.4f° in a week%n",
                    sky.at(0, 1).longitude(), sky.at(6, 1).longitude());
            // The provider's own name and data version are stamped on the
            // answer, which is how a stored chart says what computed it.
            ProviderStamp stamp = sky.provenance().provider();
            System.out.println("  stamped as " + stamp.name() + " " + stamp.version() + ", data " + stamp.dataVersion());
        }

        // ── A body it never declared ──────────────────────────────────────
        provider = new TableEphemeris();
        try (Context ctx = teistro.context(with(provider))) {
            ctx.positions(new double[] {2451545.0}, List.of(Body.SATURN));
        } catch (TeistroException error) {
            System.out.println();
            System.out.println("refused  " + error.getMessage());
            System.out.println("         and the provider was asked " + provider.calls + " times");
        }

        // ── An instant outside its coverage ───────────────────────────────
        provider = new TableEphemeris();
        try (Context ctx = teistro.context(with(provider))) {
            PositionGrid sky = ctx.positions(new double[] {2200000.0, 2451545.0}, List.of(Body.SUN));
            System.out.println();
            System.out.println("coverage 2200000 is " + ProviderCode.of(sky.at(0, 0).status()).key()
                    + " and 2451545 is " + ProviderCode.of(sky.at(1, 0).status()).key() + ":");
            System.out.println("         the provider was asked for " + provider.cells + " cell(s)");
        }

        // ── A frame it does not compute ───────────────────────────────────
        // This provider computes tropical positions and declares no native
        // frame, so the canonical one is what it answers. Ask for a sidereal
        // zodiac and the SDK does the rest, naming every step it applied,
        // which is how an engine that knows nothing about the ayanamsha can
        // still serve a Vedic chart.
        provider = new TableEphemeris(teistro);
        try (Context ctx = teistro.context(with(provider))) {
            PositionGrid tropical = ctx.positions(new double[] {2451545.0}, List.of(Body.SUN));
            Frame canonical = teistro.canonicalFrame();
            Frame lahiri = new Frame(Ayanamsha.LAHIRI, canonical.centre(), canonical.equinox(),
                    canonical.coordinates(), true, canonical.lightTime(), canonical.aberration(),
                    canonical.deflection(), canonical.nutation());
            PositionGrid sidereal = ctx.positions(new PositionRequest(TimeScale.UT1, teistro.packFrame(lahiri), true,
                    null, new double[] {2451545.0}, List.of(Body.SUN)));
            String steps = sidereal.stepsApplied().stream()
                    .map(step -> step.name() + ":" + step.implementation().key())
                    .collect(Collectors.joining(", "));
            System.out.println();
            System.out.printf(Locale.ROOT, "frames   the provider answered %.4f° tropical; a sidereal request is %.4f°%n",
                    tropical.at(0, 0).longitude(), sidereal.at(0, 0).longitude());
            System.out.println("         it refused the frame " + provider.refusals + " time(s), and the");
            System.out.println("         SDK completed it: " + steps);
        }

        // ── When the provider itself fails ────────────────────────────────
        // A checked exception arrives as a ProviderException's cause, an
        // unchecked one as itself.
        try (Context ctx = teistro.context(with(new Broken()))) {
            ctx.positions(new double[] {2451545.0}, List.of(Body.SUN));
        } catch (ProviderException error) {
            if (error.getCause() instanceof FileNotFoundException missing) {
                System.out.println();
                System.out.println("thrown   " + missing.getMessage());
                System.out.println("         the provider's own error crossed back, not just a code");
            }
        }

        // ── A refusal a user should see ───────────────────────────────────
        // Every refusal from the library carries a status a program can
        // match on and, where the boundary knows one, the field at fault and
        // a hint to act on. That is what to put in front of a person.
        try (Context ctx = teistro.context(ContextOptions.builder().profile("parashari-classical").build())) {
            ctx.keys().id("graha.SUNN");
        } catch (TeistroException error) {
            System.out.println();
            System.out.println("status   " + error.status().key());
            System.out.println("message  " + error.getMessage());
            System.out.println("detail   " + error.detail());
            System.out.println("hint     " + error.hint());
            System.out.println("         a program matches on `status`; a person reads the message and the hint");
        }
    }
}
