import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Locale;
import java.util.Optional;
import java.util.function.IntSupplier;

import com.teispace.teistro.Body;
import com.teispace.teistro.Context;
import com.teispace.teistro.ContextOptions;
import com.teispace.teistro.EphemerisProvider;
import com.teispace.teistro.PositionAnswer;
import com.teispace.teistro.PositionGrid;
import com.teispace.teistro.PositionQuery;
import com.teispace.teistro.Teistro;
import com.teispace.teistro.blob.Positions;

/**
 * The Java row of {@code docs/03-design/ffi-cost.md}: the scenario every
 * binding times, printed as one JSON line a key. {@code cargo xtask
 * measure-ffi-cost} runs it with {@code TEISTRO_LIBRARY} naming the release
 * library.
 *
 * <p>{@code call-*} asks for 1 instant and 3 bodies, {@code grid-*} for 366
 * instants and 10 bodies. {@code *-native} is a context over the library's
 * test provider and {@code *-hosted} one over a provider written here,
 * answering fixed columns. Each call reads the answer's grid size.
 * {@code decode} is the decoder alone, on the grid's blob: the binding hands
 * a consumer no call's raw bytes, so the blob is the one the Rust floor
 * wrote to the path its first argument names, the same request answered by
 * the same test provider.
 */
public final class FfiCost {
    private static final long RUN_NANOS = 50_000_000L;
    private static final int RUNS = 5;
    private static final List<Body> BODIES = List.of(
            Body.SUN, Body.MOON, Body.MERCURY, Body.VENUS, Body.MARS,
            Body.JUPITER, Body.SATURN, Body.MEAN_NODE, Body.URANUS, Body.NEPTUNE);

    private FfiCost() {}

    /** A provider written in Java: every cell a fixed longitude. */
    static final class Fixed extends EphemerisProvider {
        @Override
        public String name() {
            return "ffi-cost-in-java";
        }

        @Override
        public String version() {
            return "1";
        }

        @Override
        public List<Body> bodies() {
            return BODIES;
        }

        @Override
        public Optional<PositionAnswer> positions(PositionQuery query) {
            int cells = query.cellCount();
            double[] lon = new double[cells];
            double[] dist = new double[cells];
            double[] speed = new double[cells];
            Arrays.fill(lon, 123.25);
            Arrays.fill(dist, 1.0);
            Arrays.fill(speed, 1.0);
            return Optional.of(PositionAnswer.of(lon, new double[cells], dist).withLonSpeed(speed));
        }
    }

    private static long sink;

    private static void time(String key, IntSupplier call) {
        for (int i = 0; i < 100; i++) {
            sink += call.getAsInt();
        }
        int calls = 1;
        for (;;) {
            long start = System.nanoTime();
            for (int i = 0; i < calls; i++) {
                sink += call.getAsInt();
            }
            if (System.nanoTime() - start >= RUN_NANOS || calls >= 1 << 24) {
                break;
            }
            calls *= 2;
        }
        List<Double> runs = new ArrayList<>();
        for (int r = 0; r < RUNS; r++) {
            long start = System.nanoTime();
            for (int i = 0; i < calls; i++) {
                sink += call.getAsInt();
            }
            runs.add((System.nanoTime() - start) / 1e3 / calls);
        }
        runs.sort(null);
        System.out.println(String.format(Locale.ROOT,
                "{\"binding\":\"java\",\"key\":\"%s\",\"runs\":%d,\"calls\":%d,"
                        + "\"median_us\":%s,\"min_us\":%s,\"max_us\":%s}",
                key, RUNS, calls, runs.get(RUNS / 2), runs.get(0), runs.get(RUNS - 1)));
    }

    /**
     * Runs the scenario.
     *
     * @param args the path of the grid's blob the Rust floor wrote
     * @throws Exception when the blob cannot be read or an answer is wrong
     */
    public static void main(String[] args) throws Exception {
        if (args.length != 1) {
            throw new IllegalArgumentException("usage: FfiCost GRID, the grid's blob");
        }
        byte[] blob = Files.readAllBytes(Path.of(args[0]));
        Teistro teistro = Teistro.open();
        double[] one = {2451545.0};
        List<Body> three = List.of(Body.SUN, Body.MOON, Body.MARS);
        double[] year = new double[366];
        for (int day = 0; day < year.length; day++) {
            year[day] = 2451545.0 + day;
        }
        try (Context nativeContext = teistro.context(
                        ContextOptions.builder().profile("nepali-drik").testProvider(true).build());
                Context hosted = teistro.context(
                        ContextOptions.builder().profile("nepali-drik").provider(new Fixed()).build())) {
            // The hosted provider's answer is what it sent, so a figure is
            // never taken over a call that failed or answered something else.
            PositionGrid answered = hosted.positions(year, BODIES);
            if (answered.cellCount() != 3660 || answered.decoded().cells().lon(3659) != 123.25) {
                throw new IllegalStateException("the hosted provider did not answer the grid it was asked");
            }
            if (Positions.decode(blob).jdCount() != 366 || Positions.decode(blob).bodyCount() != 10) {
                throw new IllegalStateException("the blob at " + args[0] + " is not the grid");
            }
            time("call-native", () -> nativeContext.positions(one, three).jdCount());
            time("call-hosted", () -> hosted.positions(one, three).jdCount());
            time("grid-native", () -> nativeContext.positions(year, BODIES).jdCount());
            time("grid-hosted", () -> hosted.positions(year, BODIES).jdCount());
            time("decode", () -> (int) Positions.decode(blob).jdCount());
        }
        if (sink == 0) {
            throw new IllegalStateException("nothing was answered");
        }
    }
}
