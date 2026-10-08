import static com.teispace.teistro.Civil.at;
import static com.teispace.teistro.Civil.date;
import static com.teispace.teistro.Civil.ianaZone;

import java.util.List;
import java.util.Locale;

import com.teispace.teistro.Ayanamsha;
import com.teispace.teistro.Body;
import com.teispace.teistro.Calendar;
import com.teispace.teistro.CalendarDate;
import com.teispace.teistro.Catalogued;
import com.teispace.teistro.Context;
import com.teispace.teistro.ContextOptions;
import com.teispace.teistro.Ephemeris;
import com.teispace.teistro.Frame;
import com.teispace.teistro.Karana;
import com.teispace.teistro.Nakshatra;
import com.teispace.teistro.PositionGrid;
import com.teispace.teistro.PositionRequest;
import com.teispace.teistro.Rashi;
import com.teispace.teistro.Teistro;
import com.teispace.teistro.Tithi;
import com.teispace.teistro.TimeScale;
import com.teispace.teistro.Vara;
import com.teispace.teistro.Yoga;
import com.teispace.teistro.ZoneResolution;
import com.teispace.teistro.messages.Messages;

/**
 * The five limbs of a day, computed from the boundary alone.
 *
 * <p>A panchanga is the Hindu almanac's five parts (tithi, vara, nakshatra,
 * yoga and karana) and every one of them except the weekday is a function
 * of <b>two longitudes</b>: the Sun's and the Moon's, in the sidereal
 * zodiac. So a binding can compute a whole panchanga from {@code positions}
 * and {@code weekdayOf}, without any part of the chart layer.
 *
 * <p>The arithmetic is the SDK's own ({@code crates/panchanga/src/limb.rs}):
 *
 * <pre>
 * | limb      | from                       | divisions | each |
 * |-----------|----------------------------|-----------|------|
 * | tithi     | Moon − Sun                 | 30        | 12°  |
 * | karana    | Moon − Sun                 | 60        | 6°   |
 * | nakshatra | Moon                       | 27        | 360/27° |
 * | yoga      | Moon + Sun                 | 27        | 360/27° |
 * | vara      | the weekday                | 7         | a day |
 * </pre>
 *
 * <p>The karana is the one that is not a plain division: sixty half-tithis
 * make a lunar month and they are <b>not</b> a cycle of eleven. Kimstughna
 * opens the month, Shakuni, Chatushpada and Naga close it, and the seven
 * movable karanas repeat through everything between. {@code karanaOf}
 * below is the SDK's rule, transcribed, and the SDK holds it to the
 * corpus's own successor relation over 109 consecutive pairs.
 *
 * <p>What this example is honest about: a limb here is the one holding
 * <b>at the instant asked for</b>. A printed almanac gives the limb at
 * sunrise and the time it ends, which needs a boundary search over the
 * Moon's motion; that is what {@code crates/panchanga} does with a real
 * ephemeris, and what a binding cannot yet ask for.
 */
public final class Panchanga {
    private Panchanga() {}

    private static final double NAKSHATRA_DEG = 360.0 / 27.0;
    private static final double YOGA_DEG = 360.0 / 27.0;
    private static final double TITHI_DEG = 12.0;
    private static final double KARANA_DEG = 6.0;

    /** An angle reduced to 0 up to 360, whatever the sign it came with. */
    private static double mod360(double degrees) {
        double r = degrees % 360.0;
        return r < 0 ? r + 360.0 : r;
    }

    /**
     * The karana that a half-tithi of the lunar month is.
     *
     * <p>Transcribed from {@code crates/panchanga/src/limb.rs}. Sixty of
     * them make a month: Kimstughna opens it, Shakuni, Chatushpada and Naga
     * close it, and the seven movable karanas fill everything between, Bava
     * first from the second half of the first tithi.
     */
    private static Karana karanaOf(int halfTithi) {
        int half = Math.floorMod(halfTithi, 60);
        return switch (half) {
            case 0 -> Karana.KIMSTUGHNA;
            case 57 -> Karana.SHAKUNI;
            case 58 -> Karana.CHATUSHPADA;
            case 59 -> Karana.NAGA;
            default -> Karana.of((half - 1) % 7);
        };
    }

    /**
     * The five limbs at one instant, with the numbers behind them.
     *
     * @param sun the Sun's sidereal longitude
     * @param moon the Moon's sidereal longitude
     * @param tithi the tithi
     * @param vara the vara
     * @param nakshatra the nakshatra
     * @param yoga the yoga
     * @param karana the karana
     */
    private record Limbs(
            double sun, double moon, Tithi tithi, Vara vara, Nakshatra nakshatra, Yoga yoga, Karana karana) {
        /** How far the Moon stands ahead of the Sun, 0 to 360. */
        double elongation() {
            return mod360(moon - sun);
        }

        /** The fortnight: waxing to the full moon, waning after it. */
        String paksha() {
            return elongation() < 180.0 ? "shukla" : "krishna";
        }

        /**
         * How far through the tithi the Moon has come, as a fraction.
         *
         * <p>A printed almanac gives the <i>time</i> the tithi ends; this is
         * the fraction, which is what the fraction of a boundary search
         * would start from.
         */
        double tithiElapsed() {
            return (elongation() % TITHI_DEG) / TITHI_DEG;
        }
    }

    /**
     * The five limbs at an instant.
     *
     * <p>{@code weekday} is the ISO weekday of the <b>civil day</b> the
     * instant belongs to, which the caller has because it asked the calendar
     * for it: a vara is a property of the day, not of the moment.
     */
    private static Limbs panchangaAt(Context ctx, Teistro teistro, double instant, int weekday) {
        Frame canonical = teistro.canonicalFrame();
        Frame frame = new Frame(Ayanamsha.LAHIRI, canonical.centre(), canonical.equinox(), canonical.coordinates(),
                true, canonical.lightTime(), canonical.aberration(), canonical.deflection(), canonical.nutation());
        PositionGrid sky = ctx.positions(new PositionRequest(
                TimeScale.UT1, teistro.packFrame(frame), true, null, new double[] {instant},
                List.of(Body.SUN, Body.MOON)));
        double sun = sky.at(0, 0).longitude();
        double moon = sky.at(0, 1).longitude();
        double elongation = mod360(moon - sun);
        return new Limbs(
                sun,
                moon,
                Tithi.of((int) Math.floor(elongation / TITHI_DEG)),
                // The boundary's weekday is ISO (Monday 1 … Sunday 7) and a
                // vara counts from Sunday, so the one becomes the other by `% 7`.
                Vara.of(weekday % 7),
                Nakshatra.of((int) Math.floor(moon / NAKSHATRA_DEG)),
                Yoga.of((int) Math.floor(mod360(moon + sun) / YOGA_DEG)),
                karanaOf((int) Math.floor(elongation / KARANA_DEG)));
    }

    /** A limb's label beside the member it names. */
    private record Row(String label, Catalogued member) {}

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
            // Nepali New Year: the first day of Baisakh, BS 2082.
            CalendarDate day = date(Calendar.BIKRAM_SAMBAT, 2082, 1, 1);
            CalendarDate gregorian = ctx.calendar().convert(day, Calendar.GREGORIAN);
            // Six in the morning stands in for sunrise, which the almanac
            // would use and which needs the rise-and-set solver.
            ZoneResolution when = ctx.time().resolve(at(day, 6, 0), ianaZone("Asia/Kathmandu"));
            Limbs found = panchangaAt(ctx, teistro, when.instantJdUtc(), ctx.calendar().weekdayOf(day));

            System.out.printf(Locale.ROOT, "BS %d-%02d-%02d  (%d-%02d-%02d)  06:00 Kathmandu%n",
                    day.year(), day.month(), day.day(), gregorian.year(), gregorian.month(), gregorian.day());
            System.out.printf(Locale.ROOT, "  sun %8.4f°   moon %8.4f°   elongation %8.4f°%n",
                    found.sun(), found.moon(), found.elongation());
            System.out.println();

            for (Row row : List.of(
                    new Row("tithi", found.tithi()),
                    new Row("vara", found.vara()),
                    new Row("nakshatra", found.nakshatra()),
                    new Row("yoga", found.yoga()),
                    new Row("karana", found.karana()))) {
                Messages.EntityForms entity = ctx.intl().entity(row.member().fullKey());
                System.out.printf(Locale.ROOT, "  %-10s %-14s %-18s (%s)%n",
                        row.label(), entity.name(), entity.iast(), row.member().key());
            }
            System.out.println();
            System.out.println("  paksha     " + found.paksha());
            System.out.printf(Locale.ROOT, "  tithi is   %.1f%% elapsed at this instant%n", found.tithiElapsed() * 100);

            // The Sun on this day is the reason the year turns: BS begins at
            // the **Mesha Sankranti**, the instant the Sun enters Aries, and
            // the year's first day is the civil day that instant is reckoned
            // into. So the number worth printing is how far *past* the
            // crossing this moment is -- which is why the almanac's
            // year-start is an instant and not a date.
            //
            // A Rust example of the same scenario is what found this wrong.
            // This file said the Sun "has not quite arrived" and printed
            // 359.9023° short of Aries, when it had entered Aries two and a
            // half hours earlier: `(360 - sun) % 360` of a longitude just
            // past zero is just under 360, and reads as nearly a whole circle
            // still to go. The sign is read from the SDK rather than written
            // in, so the line stays true on any day, said in the context's
            // locale.
            String sign = ctx.intl().entity(Rashi.of((int) Math.floor(found.sun() / 30)).fullKey()).name();
            double intoSign = found.sun() % 30.0;
            System.out.printf(Locale.ROOT,
                    "  the Sun stands %.4f° into %s, so the Mesha Sankranti is about %.1f hours past --%n",
                    intoSign, sign, intoSign / 0.9856 * 24);
            System.out.println("  which is what BS 2082 is reckoned from, and why it opens today");
        }
    }
}
