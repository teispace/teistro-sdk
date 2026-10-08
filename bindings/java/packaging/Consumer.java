import static com.teispace.teistro.Civil.date;

import java.util.List;
import java.util.Locale;

import com.teispace.teistro.Body;
import com.teispace.teistro.Calendar;
import com.teispace.teistro.CalendarDate;
import com.teispace.teistro.Context;
import com.teispace.teistro.ContextOptions;
import com.teispace.teistro.Ephemeris;
import com.teispace.teistro.Teistro;

/**
 * A consumer that knows nothing but the published names.
 *
 * <p>{@code cargo xtask check-package} compiles this against the staged
 * jar, on the module path and through Maven on the class path, and runs
 * it. It asserts the same four facts the C smoke test prints, so a
 * package that loads but answers differently fails here rather than in
 * the field, and prints the library it loaded, which the gate requires to
 * be the one the jar carried. It throws rather than using {@code assert},
 * which is off unless asked for.
 */
public final class Consumer {
    private Consumer() {}

    /**
     * Runs the consumer.
     *
     * @param args unused
     */
    public static void main(String[] args) {
        Teistro teistro = Teistro.open();
        check(teistro.abiVersion() == 1, "ABI " + teistro.abiVersion());
        System.out.println("abi " + teistro.abiVersion());
        System.out.println("sdk " + teistro.sdkVersion());

        ContextOptions options = ContextOptions.builder().ephemeris(Ephemeris.BUILTIN).build();
        try (Context ctx = teistro.context(options)) {
            CalendarDate bs = ctx.calendar().convert(date(Calendar.GREGORIAN, 2015, 4, 14), Calendar.BIKRAM_SAMBAT);
            check(bs.year() == 2072 && bs.month() == 1 && bs.day() == 1, "BS " + bs);
            System.out.println("bs " + bs.year() + "-" + bs.month() + "-" + bs.day());

            double sun = ctx.positions(new double[] {2451545.0}, List.of(Body.SUN)).at(0, 0).longitude();
            check(sun >= 0.0 && sun < 360.0, "the Sun at " + sun);
            System.out.printf(Locale.ROOT, "sun %.4f%n", sun);
        }
        System.out.println("library " + teistro.path());
    }

    private static void check(boolean holds, String what) {
        if (!holds) {
            throw new AssertionError(what);
        }
    }
}
