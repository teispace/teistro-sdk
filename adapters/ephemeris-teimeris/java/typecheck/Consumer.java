import java.util.List;

import com.teispace.teistro.Body;
import com.teispace.teistro.Context;
import com.teispace.teistro.ContextOptions;
import com.teispace.teistro.Ephemeris;
import com.teispace.teistro.EphemerisChoice;
import com.teispace.teistro.Teistro;
import com.teispace.teistro.teimeris.Teimeris;
import com.teispace.teistro.teimeris.TeimerisEngine;
import com.teispace.teistro.teimeris.TeimerisEngine.TmDatetime;
import com.teispace.teistro.teimeris.TeimerisEngine.TmHousesCalc;
import com.teispace.teistro.teimeris.TeimerisEngine.TmHousesRequest;
import com.teispace.teistro.teimeris.TeimerisEngine.TmNodesApsides;
import com.teispace.teistro.teimeris.TeimerisEngine.TmPosition;
import com.teispace.teistro.teimeris.TeimerisEngine.TmPositionRequest;
import com.teispace.teistro.teimeris.TeimerisEngine.TmStar;
import com.teispace.teistro.teimeris.TeimerisEngine.TmStarQuery;
import com.teispace.teistro.teimeris.TeimerisEngine.TmVersion;

/**
 * What this package's README tells a consumer to write, held to javac at the
 * binding's own strictness.
 *
 * <p>Compiled and never run: {@code cargo xtask check-java} compiles it
 * against the adapter's module, so a README snippet that stopped compiling
 * (a renamed descriptor, a moved façade, a changed chain) fails a gate
 * rather than misleading a reader. Running it would need the engine's data,
 * which a checkout does not have. Every answer is read all the way down, so
 * a type that is wrong anywhere in it does not compile.
 */
public final class Consumer {
    private Consumer() {}

    /**
     * The README's own shape.
     *
     * @param args unused
     */
    public static void main(String[] args) {
        Teistro sdk = Teistro.open();
        // A real engine, and the SDK's own only if it is not there.
        ContextOptions options = ContextOptions.builder()
                .profile("parashari-classical")
                .ephemeris(Teimeris.builder().dataDir("./ephe").build(), EphemerisChoice.of(Ephemeris.BUILTIN))
                .build();
        try (Context sky = sdk.context(options)) {
            // The operations the SDK names, at the areas it groups them by.
            double sun = sky.positions(new double[] {2451545.0}, List.of(Body.SUN)).at(0, 0).longitude();
            System.out.println("the Sun is at " + sun);

            // And the engine's own, typed by the façade this package
            // carries: one value comes back as itself, more than one as a
            // record.
            TeimerisEngine typed = Teimeris.engine(sky.ephemeris());
            String name = typed.tmBodyName(0);
            double seconds = typed.tmDeltaT(2451545.0);
            TmVersion version = typed.tmVersion();
            long major = version.major();
            System.out.println(name + ", " + seconds + " seconds, engine " + major);

            // A struct crosses as a record, both ways.
            TmDatetime utc = typed.tmLocalToUtc(new TmDatetime(2026, 9, 13, 6, 30, 0.0), 5.75, 1);
            TmNodesApsides orbit = typed.tmNodesApsidesCalc(2461296.5, 1, 4, 0, 0, 0, null);
            double perihelion = orbit.perihelion().lon();
            System.out.println(utc.hour() + ":" + utc.minute() + " UTC, perihelion " + perihelion);

            // An array crosses as a list.
            List<Double> deltas = typed.tmDeltaTMany(List.of(2451545.0, 2461296.5));
            List<TmPosition> grid = typed.tmPositionCalcGrid(List.of(0L, 1L), List.of(2451545.0), 1, 0, null);
            List<Long> defaults = typed.tmChartDefaultBodies();
            System.out.println(deltas.get(0) + " s, the Sun at " + grid.get(0).lon() + ", "
                    + defaults.size() + " default bodies");

            // A struct that points at another takes it nested, or null.
            TmStarQuery sized = typed.tmStarQueryInitSized();
            TmStarQuery query = new TmStarQuery(sized.useMagnitude(), sized.magnitudeMin(), sized.magnitudeMax(),
                    sized.raDeg(), sized.decDeg(), sized.radiusDeg(), "Aldeb");
            List<TmStar> stars = typed.tmStarSearch(query);
            TmPosition moon = typed.tmPositionCalc(new TmPositionRequest(2451545.0, 1, 1, 0, null, 0, 0, 0));
            System.out.println(stars.size() + " stars, the Moon at " + moon.lon());

            // Cusps are as long as the house system says.
            TmHousesCalc houses = typed.tmHousesCalc(new TmHousesRequest(2451545.0, 27.7, 85.3, 0, 0));
            List<Double> cusps = houses.cusps();
            System.out.println(cusps.size() + " cusps, ascendant " + houses.outAngles().ascendant());
        }
    }
}
