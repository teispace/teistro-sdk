import static com.teispace.teistro.Civil.date;

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;

import com.teispace.teistro.Calendar;
import com.teispace.teistro.CalendarDate;
import com.teispace.teistro.Context;
import com.teispace.teistro.ContextOptions;
import com.teispace.teistro.Teistro;

/**
 * A Bikram Sambat calendar page, and why the conversions are not
 * arithmetic.
 *
 * <p>The Nepali calendar is not a formula. Its month lengths are decided by
 * where the Sun stands at the moment a month begins, so they vary year to
 * year (Baisakh is 30 or 31 or 32 days depending on the year) and the
 * authoritative table only covers BS 1970 to 2095. Outside that span the
 * SDK computes the months from the Surya Siddhanta as the text prints it.
 *
 * <p>Every date the SDK returns therefore says <b>how it was decided</b>:
 * {@code TABULAR} from the official table, {@code COMPUTED} from the
 * engine, or {@code DIVERGENT} where the two disagree and the table wins. A
 * calendar application that shows a date without showing that is hiding
 * the one thing a user might need to know.
 *
 * <p>This example builds a real calendar page: a whole BS year of month
 * lengths, then one month laid out as a grid with its Gregorian span.
 */
public final class Calendars {
    private Calendars() {}

    /** The days of the week, from the boundary's ISO numbering. */
    private static final List<String> WEEK = List.of("Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun");

    /**
     * One BS month as a calendar grid.
     *
     * <p>Every cell is a real date the SDK converted, not a number counted
     * up: a month that gains or loses a day at either end is then right by
     * construction.
     */
    private static String monthPage(Context ctx, int year, int month) {
        int length = ctx.calendar().monthLength(Calendar.BIKRAM_SAMBAT, year, month);
        CalendarDate first = date(Calendar.BIKRAM_SAMBAT, year, month, 1);
        // ISO weekday 1..7; a calendar page starts on Monday, so the first
        // of the month sits at column `weekday - 1`.
        int lead = ctx.calendar().weekdayOf(first) - 1;

        List<String> cells = new ArrayList<>();
        for (int i = 0; i < lead; i++) {
            cells.add("   ");
        }
        for (int day = 1; day <= length; day++) {
            cells.add(String.format(Locale.ROOT, "%3d", day));
        }

        List<String> lines = new ArrayList<>();
        lines.add(String.join("  ", WEEK.stream().map(name -> String.format(Locale.ROOT, "%3s", name)).toList()));
        for (int at = 0; at < cells.size(); at += 7) {
            List<String> row = cells.subList(at, Math.min(at + 7, cells.size()));
            lines.add(String.join("  ", row.stream().map(cell -> String.format(Locale.ROOT, "%3s", cell)).toList()));
        }
        return String.join("\n", lines);
    }

    /** A date with its era and how it was decided. */
    private static String described(CalendarDate day) {
        String era = day.era() == null ? "" : " " + day.era().key() + " " + day.eraYear();
        return String.format(Locale.ROOT, "%d-%02d-%02d%s [%s]",
                day.year(), day.month(), day.day(), era, day.resolution().key());
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
                .build();
        try (Context ctx = teistro.context(options)) {
            int year = 2082;
            // The Bikram Sambat messages, each rendered in the context's locale.
            var bs = ctx.intl().messages().sdk().calendar().bikramSambat();

            // ── A whole year, with its Gregorian spans ────────────────────
            System.out.println("BS " + year);
            int total = 0;
            for (int month = 1; month <= 12; month++) {
                int length = ctx.calendar().monthLength(Calendar.BIKRAM_SAMBAT, year, month);
                total += length;
                CalendarDate first = date(Calendar.BIKRAM_SAMBAT, year, month, 1);
                CalendarDate last = date(Calendar.BIKRAM_SAMBAT, year, month, length);
                CalendarDate starts = ctx.calendar().convert(first, Calendar.GREGORIAN);
                CalendarDate ends = ctx.calendar().convert(last, Calendar.GREGORIAN);
                String name = bs.monthName(month);
                System.out.printf(Locale.ROOT, "  %2d  %-10s %2d days   %d-%02d-%02d to %d-%02d-%02d%n",
                        month, name, length, starts.year(), starts.month(), starts.day(),
                        ends.year(), ends.month(), ends.day());
            }
            System.out.printf(Locale.ROOT, "      %-10s %d days in the year%n", "", total);
            // A BS year is 365 or 366 days like any solar year, but its
            // months are not: the shortest here is 29 days and the longest
            // 32, which is why a month length is asked for and never assumed.

            // ── One month as a page ──────────────────────────────────────
            System.out.println();
            System.out.println("Baisakh " + year);
            System.out.println(monthPage(ctx, year, 1));

            // ── The round trip, and what each date says about itself ──────
            System.out.println();
            CalendarDate newYear = date(Calendar.BIKRAM_SAMBAT, year, 1, 1);
            CalendarDate gregorian = ctx.calendar().convert(newYear, Calendar.GREGORIAN);
            CalendarDate back = ctx.calendar().convert(gregorian, Calendar.BIKRAM_SAMBAT);
            System.out.println("  BS   " + described(newYear));
            System.out.println("  ->   " + described(gregorian));
            System.out.println("  ->   " + described(back));
            // The library's, not a context's: a fixed day and a Julian day
            // are two spellings of one integer, and no profile or locale
            // changes the arithmetic.
            long fixed = ctx.calendar().fixedOf(newYear);
            System.out.printf(Locale.ROOT, "  fixed day %d, weekday %d, Julian day %.1f%n",
                    fixed, ctx.calendar().weekdayOf(newYear), teistro.julianDayOfFixed(fixed));

            // ── Inside the table, and outside it ──────────────────────────
            // A date a caller *states* is always `DEFINED`: it is what was
            // asked for. A date the SDK *returns* says how it was decided, so
            // the resolution to read is the one on the answer.
            System.out.println();
            for (int asked : new int[] {2082, 2200, 1960}) {
                CalendarDate began = ctx.calendar().convert(
                        date(Calendar.BIKRAM_SAMBAT, asked, 1, 1), Calendar.GREGORIAN);
                CalendarDate answer = ctx.calendar().convert(began, Calendar.BIKRAM_SAMBAT);
                System.out.printf(Locale.ROOT, "  BS %d began %d-%02d-%02d, and the answer is [%s]%n",
                        asked, began.year(), began.month(), began.day(), answer.resolution().key());
            }
            System.out.println("       the official table runs BS 1970 to 2095; on either side"
                    + " the SDK's own engine answers, and says so");

            // ── The typed message accessors ───────────────────────────────
            // A date rendered for a reader goes through the locale, not
            // through string concatenation: the key is spelled once, in the
            // generator, and the parameters are typed.
            System.out.println();
            System.out.println("  rendered  " + bs.date().long_(1, bs.monthName(1), year));
        }
    }
}
