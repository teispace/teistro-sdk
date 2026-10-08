import static com.teispace.teistro.Civil.at;
import static com.teispace.teistro.Civil.date;
import static com.teispace.teistro.Civil.ianaZone;

import java.util.List;
import java.util.Locale;

import com.teispace.teistro.Body;
import com.teispace.teistro.Calendar;
import com.teispace.teistro.CalendarDate;
import com.teispace.teistro.Context;
import com.teispace.teistro.ContextOptions;
import com.teispace.teistro.Ephemeris;
import com.teispace.teistro.PositionGrid;
import com.teispace.teistro.Teistro;
import com.teispace.teistro.ZoneResolution;
import com.teispace.teistro.messages.Messages;

/**
 * The README's example, kept honest: {@code cargo xtask check-parity} runs
 * this file beside the other bindings' and compares what it prints.
 */
public final class Quickstart {
    private Quickstart() {}

    /**
     * Runs the example.
     *
     * @param args unused
     */
    public static void main(String[] args) {
        Teistro teistro = Teistro.open();
        // The SDK's version is what a log line wants. The ABI -- the C
        // boundary's revision -- was checked when the library opened: one
        // this package was not generated against is refused there, so
        // `abiVersion()` is for a bug report rather than for every run.
        System.out.println("Teistro " + teistro.sdkVersion());

        ContextOptions options = ContextOptions.builder()
                .profile("nepali-default")
                .locale("ne-Deva-NP")
                .ephemeris(Ephemeris.BUILTIN)
                .build();
        try (Context ctx = teistro.context(options)) {
            // 14 April 2015 is 1 Baisakh 2072 BS.
            CalendarDate bs = ctx.calendar().convert(date(Calendar.GREGORIAN, 2015, 4, 14), Calendar.BIKRAM_SAMBAT);
            String era = bs.era() != null ? bs.era().fullKey() : "";
            System.out.println(bs.year() + "-" + bs.month() + "-" + bs.day() + " " + era);

            // A Kathmandu birth time, with the metadata a stored chart keeps.
            ZoneResolution resolved = ctx.time().resolve(
                    at(date(Calendar.GREGORIAN, 1986, 1, 1), 0, 20), ianaZone("Asia/Kathmandu"));
            System.out.printf(Locale.ROOT, "JD %.6f UTC, %d s, tzdb %s%n",
                    resolved.instantJdUtc(), resolved.offsetSeconds(), resolved.tzdbVersion());

            // The Sun and the Moon at J2000, in the SDK's canonical frame.
            PositionGrid sky = ctx.positions(new double[] {2451545.0}, List.of(Body.SUN, Body.MOON));
            System.out.printf(Locale.ROOT, "the Sun at %.4f degrees%n", sky.at(0, 0).longitude());

            // A message in the context's locale, by its typed accessor, and
            // an entity's name in that locale.
            System.out.println(ctx.intl().messages().sdk().reason().grahaInBhava(7, Messages.GrahaKey.JUPITER));
            Messages.EntityForms sun = ctx.intl().entity("graha.SUN");
            System.out.println(sun.name() + " " + sun.glyph());
        }
    }
}
