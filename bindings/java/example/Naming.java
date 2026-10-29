import java.util.ArrayList;
import java.util.List;

import com.teispace.teistro.Context;
import com.teispace.teistro.ContextOptions;
import com.teispace.teistro.LoadedBook;
import com.teispace.teistro.Nakshatra;
import com.teispace.teistro.NameCheck;
import com.teispace.teistro.NameChecked;
import com.teispace.teistro.NameClauseRow;
import com.teispace.teistro.NameClauses;
import com.teispace.teistro.NameInitial;
import com.teispace.teistro.NameRead;
import com.teispace.teistro.NameReading;
import com.teispace.teistro.Names;
import com.teispace.teistro.PadaSyllable;
import com.teispace.teistro.RefusedName;
import com.teispace.teistro.Sex;
import com.teispace.teistro.SpelledName;
import com.teispace.teistro.Suggested;
import com.teispace.teistro.Suggestions;
import com.teispace.teistro.Teistro;
import com.teispace.teistro.messages.Messages;
import com.teispace.teistro.request.Birth;
import com.teispace.teistro.request.Book;
import com.teispace.teistro.request.BookName;
import com.teispace.teistro.request.LatinName;
import com.teispace.teistro.request.NameRules;
import com.teispace.teistro.request.NamingRules;
import com.teispace.teistro.request.SuggestFilter;

/**
 * Naming a child (nāmakaraṇa): the syllable a birth pada names a child by,
 * names checked against it, a Latin spelling read every way it can be, a
 * family's own book of names searched, and the clauses a name in daily use
 * is held to, said in English and in Nepali. {@code cargo xtask
 * check-parity} runs this file beside the other bindings' and compares
 * what it prints ({@code 03-design/namakarana.md}).
 *
 * <p>What it is really teaching:
 *
 * <ol>
 *   <li><b>Facts, not a score.</b> A check says whether a name begins in
 *       the birth star's row, in the pada's own cell, or by the patro's
 *       substitute letter, and what each clause of the gṛhyasūtras makes
 *       of it. Nothing adds them up: which of them matters is the
 *       family's.
 *   <li><b>A Latin spelling is read every way it can be.</b> {@code
 *       CASUAL} answers each cell a spelling reaches and says it is
 *       ambiguous, rather than choosing one.
 *   <li><b>The SDK ships no names.</b> A book is the consumer's, loaded
 *       once and named by its id; a name the cakra cannot read is listed,
 *       not dropped.
 *   <li><b>A clause is a key; the locale says it.</b> {@code sdk.naming}'s
 *       messages turn each key into words, so one answer reads in every
 *       locale.
 * </ol>
 *
 * <p>It reads no sky, so it names no ephemeris.
 */
public final class Naming {
    private Naming() {}

    /** A flag as every binding prints it. */
    private static String yes(boolean flag) {
        return flag ? "yes" : "no";
    }

    /** A syllable's star, or Abhijit's row, which is none of the 27. */
    private static String star(Nakshatra nakshatra) {
        return nakshatra == null ? "Abhijit" : nakshatra.fullKey();
    }

    /**
     * Runs the example.
     *
     * @param args unused
     */
    public static void main(String[] args) {
        Teistro teistro = Teistro.open();
        try (Context ctx = teistro.context(ContextOptions.builder().locale("en-Latn").build())) {
            // ── The birth's syllable ───────────────────────────────────
            // The Moon in Ardra's third pada. Its syllable is ङ, which no
            // name begins with, so a patro printed in Nepal glosses it with ग: a
            // substitute, listed after the cell's own letter.
            Birth birth = new Birth(Nakshatra.ARDRA, 3);
            Names names = ctx.naming().of(birth, null);
            PadaSyllable syllable = names.syllable();
            System.out.println(star(syllable.nakshatra()) + " " + birth.pada() + ": " + syllable.devanagari() + " "
                    + syllable.iast() + ", deity " + names.deity().fullKey());
            List<String> initials = new ArrayList<>();
            for (NameInitial initial : syllable.initials()) {
                initials.add(initial.devanagari() + " " + initial.by().name());
            }
            System.out.println("initials " + String.join(", ", initials));

            // ── Names checked against it ───────────────────────────────
            // Each name is answered on its own: ऐ is no cell of the cakra,
            // and its refusal refuses no other name.
            List<String> asked = List.of("गणेश", "चूडामणि", "ऐश्वर्य");
            List<NameChecked> checked = ctx.naming().check(asked, birth, null);
            for (int i = 0; i < asked.size(); i += 1) {
                NameCheck c = checked.get(i).check();
                System.out.println(c == null
                        ? asked.get(i) + ": refused, " + checked.get(i).refused().message()
                        : asked.get(i) + ": same star " + yes(c.sameStar()) + ", same pada " + yes(c.samePada())
                                + ", by substitute " + yes(c.bySubstitute()) + ", " + c.clauses().syllables()
                                + " syllables");
            }

            // ── A Latin spelling, read every way ───────────────────────
            List<String> spellings = List.of("Krishna", "Shiva");
            NamingRules casual = NamingRules.builder().name(new NameRules(null, LatinName.CASUAL)).build();
            List<NameRead> read = ctx.naming().read(spellings, casual);
            for (int i = 0; i < spellings.size(); i += 1) {
                NameReading reading = read.get(i).reading();
                List<String> cells = new ArrayList<>();
                for (SpelledName one : reading.readings()) {
                    cells.add(one.spelled() + " " + star(one.syllable().nakshatra()) + " " + one.syllable().quarter());
                }
                System.out.println(spellings.get(i) + ": ambiguous " + yes(reading.ambiguous()) + ", "
                        + String.join(" | ", cells));
            }

            // ── A family's own book ────────────────────────────────────
            // The SDK ships no names. A book is loaded once into the
            // context and named after by its id; औषधि begins with औ, no
            // cell, and is listed.
            LoadedBook loaded = ctx.naming().load(new Book(List.of(
                    BookName.builder().devanagari("गणेश").sex(Sex.MALE).build(),
                    BookName.builder().devanagari("गीता").sex(Sex.FEMALE).build(),
                    new BookName("गोपाल"),
                    new BookName("औषधि"))));
            List<String> refused = new ArrayList<>();
            for (RefusedName one : loaded.refused()) {
                refused.add(one.index() + " " + one.error().field());
            }
            System.out.println("a book of " + loaded.names() + " names, refused: " + String.join(", ", refused));
            // A boy's names: the book's boys and the names it gives no sex.
            Suggestions found = ctx.naming().suggest(loaded.book(), birth,
                    SuggestFilter.builder().sex(Sex.MALE).build(), null);
            List<String> suggested = new ArrayList<>();
            for (Suggested one : found.names()) {
                suggested.add(one.reading().spelled() + " " + one.by().name());
            }
            System.out.println("suggested for a boy: " + String.join(", ", suggested));

            // ── Said in each locale ────────────────────────────────────
            // The answer holds keys; `sdk.naming`'s messages say them.
            NameCheck ganesh = ctx.naming().check(List.of("गणेश"), birth, null).get(0).check();
            for (String locale : List.of("en-Latn", "ne-Deva-NP")) {
                ctx.intl().setLocale(locale);
                Messages.Sdk.SdkNaming say = ctx.intl().messages().sdk().naming();
                System.out.println(locale);
                System.out.println("  " + say.pada(Messages.NakshatraKey.ARDRA, birth.pada(), syllable.devanagari()));
                NameClauses clauses = ganesh.clauses();
                System.out.println("  गणेश: " + say.syllables(clauses.syllables()) + ", "
                        + say.counted(clauses.counted().key()));
                for (NameClauseRow row : clauses.rows()) {
                    System.out.println("    " + say.clause(row.clause().key()) + ": " + say.holds(yes(row.holds()))
                            + " (" + say.source(row.source().name()) + ")");
                }
                for (Suggested one : found.names()) {
                    System.out.println("  " + one.reading().spelled() + ": " + say.by(one.by().name()));
                }
            }
        }
    }
}
