package com.teispace.teistro;

import java.io.FileDescriptor;
import java.io.FileOutputStream;
import java.io.PrintStream;
import java.math.BigDecimal;
import java.math.RoundingMode;
import java.nio.charset.StandardCharsets;
import java.time.LocalDate;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Objects;
import java.util.TreeMap;
import java.util.function.Function;

import com.teispace.teistro.blob.IntlRender;
import com.teispace.teistro.messages.Messages;
import com.teispace.teistro.record.Deviation;
import com.teispace.teistro.record.Provenance;

/**
 * One scenario through the Java binding, printed as the parity report.
 *
 * <p>{@code key<TAB>value} lines, sorted by key. {@code cargo xtask check-parity}
 * runs this beside {@code bindings/python/parity.py},
 * {@code bindings/node/parity.mjs} and {@code bindings/dart/bin/parity.dart}
 * and compares what they print, so a difference between the bindings' layers
 * is a failed gate rather than something a reader has to notice.
 *
 * <p>This is a port of the Python runner, section by section and key by key.
 * Every value is what this binding's own surface gives, written the way the
 * Python runner writes it: an enum as the key it spells, a number to nine
 * decimals as Python's {@code f"{value:.9f}"} formats it (an integral value
 * written plainly), a flag as Python's {@code put} or {@code int(flag)} writes
 * it, a JSON section as its FNV-1a hash over its UTF-8 bytes, and the lines
 * sorted by code point as Python's {@code sorted()} sorts strings.
 *
 * <p>The keys the Java binding cannot yet answer are not printed, and each
 * would be a gap in the binding, so none is left.
 */
public final class ParityRunner {
    private ParityRunner() {}

    /** The report, sorted by code point as Python's {@code sorted()} sorts strings. */
    private static final Map<String, String> REPORT = new TreeMap<>(ParityRunner::byCodePoint);

    /** A member of the surface, referenced and never called. */
    @FunctionalInterface
    private interface Reference {
        void touch();
    }

    /**
     * Walks the scenario and prints the report, sorted by key, to standard
     * output in UTF-8.
     *
     * @param args unused
     */
    public static void main(String[] args) {
        try {
            run();
        } catch (Throwable failure) {
            failure.printStackTrace();
            System.exit(1);
        }
        StringBuilder text = new StringBuilder();
        for (Map.Entry<String, String> line : REPORT.entrySet()) {
            text.append(line.getKey()).append('\t').append(line.getValue()).append('\n');
        }
        PrintStream out = new PrintStream(new FileOutputStream(FileDescriptor.out), false, StandardCharsets.UTF_8);
        out.print(text);
        out.flush();
        if (out.checkError()) {
            System.exit(1);
        }
    }

    // ── Formatting, as the Python runner formats ─────────────────────────

    /** Python's {@code sorted()} on {@code str}: by code point, not by UTF-16 unit. */
    static int byCodePoint(String a, String b) {
        int i = 0;
        int j = 0;
        while (i < a.length() && j < b.length()) {
            int x = a.codePointAt(i);
            int y = b.codePointAt(j);
            if (x != y) {
                return Integer.compare(x, y);
            }
            i += Character.charCount(x);
            j += Character.charCount(y);
        }
        return Boolean.compare(i < a.length(), j < b.length());
    }

    /** Python's {@code number} on an int: {@code str(value)}. */
    static String number(long value) {
        return Long.toString(value);
    }

    /**
     * Python's {@code number} on a float: an integral value below 1e15 as
     * {@code str(int(value))} (so negative zero is {@code 0}), anything else
     * as {@code f"{value:.9f}"}.
     */
    static String number(double value) {
        if (value == Math.rint(value) && Math.abs(value) < 1e15) {
            return Long.toString((long) value);
        }
        return fixed(value, 9);
    }

    /**
     * Python's {@code f"{value:.Nf}"}: the double's exact binary value
     * rounded half to even, never an exponent, and the sign kept even where
     * the digits round to zero ({@code -0.000000000}).
     */
    static String fixed(double value, int places) {
        if (!Double.isFinite(value)) {
            return Double.isNaN(value) ? "nan" : value > 0 ? "inf" : "-inf";
        }
        String digits = new BigDecimal(Math.abs(value)).setScale(places, RoundingMode.HALF_EVEN).toPlainString();
        boolean negative = value < 0 || (value == 0 && 1 / value < 0);
        return negative ? "-" + digits : digits;
    }

    /**
     * Python's {@code repr(float)}, which {@code str()} and an f-string write:
     * the shortest digits that read back (what {@link Double#toString} gives
     * from JDK 19), fixed where the decimal point falls in (-4, 16] and an
     * exponent of at least two digits otherwise.
     */
    static String repr(double value) {
        if (Double.isNaN(value)) {
            return "nan";
        }
        if (Double.isInfinite(value)) {
            return value > 0 ? "inf" : "-inf";
        }
        if (value == 0) {
            return 1 / value < 0 ? "-0.0" : "0.0";
        }
        BigDecimal shortest = new BigDecimal(Double.toString(Math.abs(value))).stripTrailingZeros();
        String digits = shortest.unscaledValue().toString();
        int point = digits.length() - shortest.scale();
        String sign = value < 0 ? "-" : "";
        if (-4 < point && point <= 16) {
            if (point <= 0) {
                return sign + "0." + "0".repeat(-point) + digits;
            }
            if (point >= digits.length()) {
                return sign + digits + "0".repeat(point - digits.length()) + ".0";
            }
            return sign + digits.substring(0, point) + "." + digits.substring(point);
        }
        String mantissa = digits.length() == 1 ? digits : digits.charAt(0) + "." + digits.substring(1);
        int exponent = point - 1;
        return sign + mantissa + "e" + (exponent < 0 ? "-" : "+") + String.format(Locale.ROOT, "%02d", Math.abs(exponent));
    }

    /** Python's {@code str()} of a value JSON read: what an f-string writes for it. */
    static String py(Object value) {
        if (value == null) {
            return "None";
        }
        if (value instanceof Boolean flag) {
            return flag ? "True" : "False";
        }
        if (value instanceof Double real) {
            return repr(real);
        }
        if (value instanceof Number whole) {
            return Long.toString(whole.longValue());
        }
        if (value instanceof String text) {
            return text;
        }
        return pyRepr(value);
    }

    /** Python's {@code repr()} of a value JSON read, as {@code str()} of a container writes its items. */
    private static String pyRepr(Object value) {
        if (value instanceof String text) {
            return "'" + text.replace("\\", "\\\\").replace("'", "\\'") + "'";
        }
        if (value instanceof List<?> items) {
            List<String> said = new ArrayList<>();
            for (Object item : items) {
                said.add(pyRepr(item));
            }
            return "[" + String.join(", ", said) + "]";
        }
        if (value instanceof Map<?, ?> fields) {
            List<String> said = new ArrayList<>();
            for (Map.Entry<?, ?> field : fields.entrySet()) {
                said.add(pyRepr(field.getKey()) + ": " + pyRepr(field.getValue()));
            }
            return "{" + String.join(", ", said) + "}";
        }
        return py(value);
    }

    /** FNV-1a over UTF-8 bytes, so a JSON section can be compared without a parser. */
    static String fnv(String text) {
        int digest = 0x811C9DC5;
        for (byte b : text.getBytes(StandardCharsets.UTF_8)) {
            digest = (digest ^ (b & 0xFF)) * 0x01000193;
        }
        return String.format(Locale.ROOT, "%08x", digest);
    }

    /**
     * Python's {@code put}: a flag as {@code true} or {@code false}, an int
     * plainly, a float by {@link #number(double)}, none as {@code null} and
     * anything else as its text.
     */
    private static void put(String key, Object value) {
        String text;
        if (value == null) {
            text = "null";
        } else if (value instanceof Boolean flag) {
            text = flag ? "true" : "false";
        } else if (value instanceof Double real) {
            text = number(real.doubleValue());
        } else if (value instanceof Float real) {
            text = number(real.doubleValue());
        } else if (value instanceof Number whole) {
            text = Long.toString(whole.longValue());
        } else {
            text = value.toString();
        }
        REPORT.put(key, text);
    }

    /** Python's {@code int(flag)}, as a runner writes a clause: 1 held, 0 not. */
    private static String flag(boolean value) {
        return value ? "1" : "0";
    }

    /** Python's {@code str(flag).lower()}. */
    private static String lower(boolean value) {
        return value ? "true" : "false";
    }

    /** Items joined, each as {@code said} writes it. */
    private static <T> String join(String separator, List<T> items, Function<? super T, String> said) {
        List<String> out = new ArrayList<>(items.size());
        for (T item : items) {
            out.add(said.apply(item));
        }
        return String.join(separator, out);
    }

    /** Python's {@code text or "-"}. */
    private static String orDash(String text) {
        return text.isEmpty() ? "-" : text;
    }

    /** Python's {@code text or "none"}. */
    private static String orNone(String text) {
        return text.isEmpty() ? "none" : text;
    }

    /** Graha full keys joined by commas, or {@code -} for none. */
    private static String grahaKeys(List<Graha> grahas) {
        return orDash(join(",", grahas, Graha::fullKey));
    }

    /** Integers joined by a separator. */
    private static String ints(String separator, List<Integer> values) {
        return join(separator, values, value -> Integer.toString(value));
    }

    /** Doubles, each by {@link #number(double)}, joined by a separator. */
    private static String numbers(String separator, List<Double> values) {
        return join(separator, values, ParityRunner::number);
    }

    /** Python's truthiness of a value JSON read. */
    private static boolean truthy(Object value) {
        return switch (value) {
            case null -> false;
            case Boolean flag -> flag;
            case String text -> !text.isEmpty();
            case Double real -> real != 0;
            case Number whole -> whole.longValue() != 0;
            case List<?> items -> !items.isEmpty();
            case Map<?, ?> fields -> !fields.isEmpty();
            default -> true;
        };
    }

    /** Python's {@code record[a][b]...} over JSON: a missing field is a failure, as a KeyError is. */
    private static Object dig(Object record, String... path) {
        Object at = record;
        for (String name : path) {
            if (!(at instanceof Map<?, ?> fields) || !fields.containsKey(name)) {
                throw new IllegalStateException("the JSON has no `" + name + "` along " + String.join(".", path));
            }
            at = fields.get(name);
        }
        return at;
    }

    /** A JSON list, or a failure. */
    private static List<?> list(Object value) {
        if (value instanceof List<?> items) {
            return items;
        }
        throw new IllegalStateException("expected a JSON list, found " + value);
    }

    /** A request object, its fields in the order written. */
    private static Map<String, Object> map(Object... fields) {
        Map<String, Object> out = new LinkedHashMap<>();
        for (int at = 0; at < fields.length; at += 2) {
            out.put((String) fields[at], fields[at + 1]);
        }
        return out;
    }

    /** Python's {@code date(Calendar.GREGORIAN, year, month, day)}. */
    private static CalendarDate date(int year, int month, int day) {
        return new CalendarDate(Calendar.GREGORIAN, null, year, 0, month, day, Resolution.DEFINED, 0, 0);
    }

    /** The build's text, as Python's {@code BuildInfo.text} reads it. */
    private static String text(Object value) {
        return value == null ? "" : String.valueOf(value);
    }

    /** The build's number, as Python's {@code BuildInfo.number} reads it. */
    private static long whole(Object value) {
        return value instanceof Number n ? n.longValue() : 0;
    }

    // ── The shared readings ───────────────────────────────────────────────

    /** A natal point as every runner prints it: {@code LAGNA}, or the graha's full key. */
    private static String natalKey(NatalPoint point) {
        return point.graha() == null ? "LAGNA" : point.graha().fullKey();
    }

    /** A harmonic point as every runner prints it: a graha's full key, or the angle's name. */
    private static String harmonicKey(HarmonicPoint point) {
        return point.graha() == null ? point.point() : point.graha().fullKey();
    }

    /**
     * A koota's reading as every runner prints it: its fields in serde's
     * order, a member by its full key, a flag as 0 or 1 and no dosha as
     * {@code NONE}.
     */
    private static String readingText(KootaReading reading) {
        if (reading instanceof VashyaKoota k) {
            return k.relation().key();
        }
        if (reading instanceof TaraKoota k) {
            return k.brideToGroom() + " " + k.groomToBride();
        }
        if (reading instanceof YoniKoota k) {
            return k.bride().fullKey() + " " + k.groom().fullKey() + " " + k.relation().key();
        }
        if (reading instanceof MaitriKoota k) {
            return k.bride().fullKey() + " " + k.groom().fullKey() + " " + k.relation().key() + " " + flag(k.lifted());
        }
        if (reading instanceof GanaKoota k) {
            return k.bride().fullKey() + " " + k.groom().fullKey() + " " + flag(k.dosha()) + " " + flag(k.lifted());
        }
        if (reading instanceof BhakootKoota k) {
            BhakootExceptions e = k.exceptions();
            String dosha = k.dosha() == null ? "NONE" : k.dosha().key();
            return String.join(" ", String.valueOf(k.apart()), dosha, flag(e.oneLord()), flag(e.lordsFriends()),
                    flag(e.navamshaLordsFriends()), flag(e.taraPure()), flag(e.vashya()), flag(k.lifted()));
        }
        if (reading instanceof NadiKoota k) {
            return k.bride().fullKey() + " " + k.groom().fullKey() + " " + flag(k.dosha()) + " " + flag(k.lifted());
        }
        if (reading instanceof VarnaKoota k) {
            return k.bride().fullKey() + " " + k.groom().fullKey();
        }
        throw new IllegalStateException("a koota this runner does not know: " + reading);
    }

    /**
     * A consideration's reading as every runner prints it: its fields in
     * serde's order, a member by its full key (a boundary enum by its key)
     * and a flag as 0 or 1.
     */
    private static String poruthamText(PoruthamReading reading) {
        if (reading instanceof DhinamPorutham p) {
            return p.count() + " " + p.rule().key();
        }
        if (reading instanceof GanamPorutham p) {
            return p.bride().fullKey() + " " + p.groom().fullKey() + " " + flag(p.diminished());
        }
        if (reading instanceof YoniPorutham p) {
            return p.bride().fullKey() + " " + p.groom().fullKey() + " " + flag(p.hostile());
        }
        if (reading instanceof RasiPorutham p) {
            return String.valueOf(p.apart());
        }
        if (reading instanceof RasyadhipathiPorutham p) {
            return p.bride().fullKey() + " " + p.groom().fullKey() + " " + flag(p.brideCallsFriend()) + " "
                    + flag(p.groomCallsFriend());
        }
        if (reading instanceof VasyamPorutham p) {
            return flag(p.brideToGroom()) + " " + flag(p.groomToBride());
        }
        if (reading instanceof RajjuPorutham p) {
            return p.bride().key() + " " + p.groom().key();
        }
        if (reading instanceof VedhaiPorutham p) {
            return flag(p.pierced());
        }
        if (reading instanceof MahendraPorutham p) {
            return String.valueOf(p.count());
        }
        if (reading instanceof DeerghaPorutham p) {
            return String.valueOf(p.count());
        }
        throw new IllegalStateException("a porutham this runner does not know: " + reading);
    }

    /** A pair as every runner writes it, the degrees rounded alike. */
    private static String pairSaid(TajikaBetween p) {
        String yoga = p.yoga() != null ? p.yoga().key() : "-";
        return p.faster().fullKey() + ">" + p.slower().fullKey() + ":" + p.drishti().key() + ":" + yoga + ":"
                + fixed(p.apartDeg(), 6);
    }

    /** Afflictions joined by {@code +}, or {@code none}. */
    private static String clausesSaid(List<Affliction> clauses) {
        return clauses == null || clauses.isEmpty() ? "none" : join("+", clauses, Affliction::key);
    }

    /** A yoga that held, and what made it; {@code -} wherever there is none. */
    private static String heldSaid(HeldYearYoga h) {
        return String.join(":",
                h.yoga().key(),
                h.through() != null ? h.through().fullKey() : "-",
                h.entering() != null ? h.entering().fullKey() : "-",
                h.between() != null ? "pair" : "-",
                h.legs() != null ? join("/", h.legs(), ParityRunner::pairSaid) : "-",
                h.afflictions() != null
                        ? clausesSaid(h.afflictions().lagnesha()) + "/" + clausesSaid(h.afflictions().karyesha())
                        : "-");
    }

    /** Devotions as {@code graha:deity|deity:verse:withKetu}, joined by commas. */
    private static String devotionsSaid(List<Devotion> devotions) {
        return orDash(join(",", devotions, d -> d.graha().fullKey() + ":" + String.join("|", d.deities()) + ":"
                + d.verse() + ":" + lower(d.withKetu())));
    }

    /** A chart's remedies, each row under {@code at}. */
    private static void putRemedies(String at, Remedies rm) {
        Functional fn = rm.functional();
        Map<String, Object> rules = rm.rules();
        put(at, py(dig(rules, "functional", "scheme")) + " " + py(dig(rules, "shanti", "rik")) + " "
                + py(dig(rules, "devata", "sunWithKetu")) + " " + fn.lagna().fullKey() + " "
                + grahaKeys(fn.yogakarakas()) + " " + grahaKeys(fn.marakas()) + " " + fn.badhaka().house() + ":"
                + fn.badhaka().lord().fullKey());
        for (FunctionalRow row : fn.rows()) {
            put(at + "-nature-" + row.graha().fullKey(), orDash(ints(",", row.houses())) + " "
                    + orDash(join(",", row.clauses(), c -> c.kind() + ":" + c.house())) + " " + row.nature());
        }
        for (RemedySubject one : rm.subjects().subjects()) {
            put(at + "-subject-" + one.graha().fullKey(), orDash(String.join(",", one.reasons())));
        }
        AntardashaShanti running = rm.subjects().antardasha();
        if (running != null) {
            DashaShanti ds = running.shanti();
            String holds = orDash(join(",", running.holds(), h -> h == null ? "-" : lower(h)));
            put(at + "-antardasha", ds.mahadasha().fullKey() + " " + ds.antardasha().fullKey() + " " + ds.chapter()
                    + " " + ds.verses() + " " + ds.page() + " " + orDash(String.join(",", ds.conditions())) + " "
                    + orDash(String.join(",", ds.remedies())) + " " + holds);
        }
        for (Shanti sh : rm.shantis()) {
            String substance = sh.substance() == null || sh.substance().isEmpty() ? "-" : sh.substance();
            put(at + "-shanti-" + sh.graha().fullKey(), sh.image() + " " + sh.japaThousands() + " " + sh.samidh()
                    + " " + sh.food() + " " + sh.dakshina() + " " + sh.gem() + " " + substance + " "
                    + (sh.direction() != null ? sh.direction().fullKey() : "-") + " " + sh.mandala() + " " + sh.rik());
        }
        IshtaDevatas dv = rm.ishtaDevata();
        put(at + "-devata", dv.atmakaraka().fullKey() + " " + dv.karakamsha().fullKey());
        Map<String, IshtaDevata> devatas = new LinkedHashMap<>();
        devatas.put("rasi", dv.inRasi());
        devatas.put("navamsha", dv.inNavamsha());
        for (Map.Entry<String, IshtaDevata> named : devatas.entrySet()) {
            IshtaDevata one = named.getValue();
            put(at + "-devata-" + named.getKey(), py(dig(one.rules(), "sunWithKetu")) + " " + one.sign().fullKey()
                    + " " + devotionsSaid(one.devotions()) + " " + grahaKeys(one.minor()));
        }
        AmatyaDevatas am = dv.amatya();
        put(at + "-amatya", am.graha().fullKey() + " " + am.amsha().fullKey());
        Map<String, AmatyaDevata> amatyas = new LinkedHashMap<>();
        amatyas.put("rasi", am.inRasi());
        amatyas.put("navamsha", am.inNavamsha());
        for (Map.Entry<String, AmatyaDevata> named : amatyas.entrySet()) {
            AmatyaDevata reading = named.getValue();
            put(at + "-amatya-" + named.getKey(), reading.twelfth().sign().fullKey() + " "
                    + devotionsSaid(reading.twelfth().devotions()) + " " + grahaKeys(reading.twelfth().minor()) + " "
                    + reading.sign().fullKey() + " " + reading.house() + " " + devotionsSaid(reading.joined()));
        }
    }

    /** One matter's Tajika yogas, each row under {@code at}. */
    private static void putMatter(String at, TajikaMatter matter) {
        put(at, matter.sign().fullKey() + " " + matter.lagnesha().fullKey() + ">" + matter.karyesha().fullKey() + " "
                + lower(matter.sameLord()));
        put(at + "-pair", matter.between() != null ? pairSaid(matter.between()) : "-");
        put(at + "-unanswered", join(",", matter.unanswered(), YearYoga::key));
        put(at + "-held", join(" ", matter.held(), ParityRunner::heldSaid));
    }

    /** Pairs in antiscion as every runner prints them: their count, then each pair's planets, side, gap and orb. */
    private static void putAntiscionRows(String key, List<AntiscionRow> rows) {
        put(key + "-count", String.valueOf(rows.size()));
        for (int n = 0; n < rows.size(); n += 1) {
            AntiscionRow row = rows.get(n);
            put(key + "-" + n, row.first().fullKey() + " " + row.second().fullKey() + " " + flag(row.contrary()) + " "
                    + number(row.apartDeg()) + " " + number(row.orbDeg()));
        }
    }

    /** A muhurta answer's counts, hash and every window, as every runner prints them. */
    private static void putMuhurta(String prefix, MuhurtaAnswer answer) {
        put(prefix + "-counts", answer.windows().size() + " " + answer.closed().size() + " " + answer.daysJudged() + " "
                + answer.daysCut() + " " + answer.windowsBlackedOut() + " " + answer.ranking());
        put(prefix + "-hash", answer.provenance().contentHash());
        for (int k = 0; k < answer.windows().size(); k += 1) {
            MuhurtaWindow window = answer.windows().get(k);
            put(prefix + "-" + k, number(window.at().fromJd()) + " " + number(window.at().toJd()));
            put(prefix + "-" + k + "-clauses", join(" ", window.clauses(), clause -> clause.kind().clause()));
            put(prefix + "-" + k + "-bars",
                    orNone(join(" ", window.barredBy(), bar -> bar.tag() != null ? bar.tag() : bar.clause().clause())));
            List<String> placed = new ArrayList<>();
            for (MuhurtaClause c : window.clauses()) {
                if (c.kind() instanceof MuhurtaClauseKind.UnwantedPlacementClause unwanted) {
                    placed.add(unwanted.house() + ":" + join(",", unwanted.by(), Graha::fullKey));
                }
            }
            put(prefix + "-" + k + "-placed", orNone(String.join(" ", placed)));
            MuhurtaScore score = window.score();
            put(prefix + "-" + k + "-score", score == null
                    ? "none"
                    : String.join(" ", String.valueOf(score.value()),
                            score.cappedAt() == null ? "none" : String.valueOf(score.cappedAt()),
                            orNone(join(" ", score.factors(), f -> f.dimension() + ":" + f.weight() + ":"
                                    + (f.graha() == null ? "none" : f.graha().fullKey())))));
        }
        for (int j = 0; j < answer.closed().size(); j += 1) {
            ClosedDay day = answer.closed().get(j);
            put(prefix + "-closed-" + j, day.date().month() + "-" + day.date().day() + " "
                    + orNone(join(" ", day.by(), BlackoutKind::fullKey)));
        }
    }

    /** A festival answer's counts, hash, every observance and every Ekadashi fast, as every runner prints them. */
    private static void putFestivals(String prefix, FestivalAnswer answer) {
        put(prefix + "-counts", answer.observances().size() + " " + answer.unjudged().size());
        put(prefix + "-hash", answer.provenance().contentHash());
        for (int k = 0; k < answer.observances().size(); k += 1) {
            FestivalObservance observance = answer.observances().get(k);
            FestivalDecided decided = observance.decidedBy();
            String by;
            if ("GUARD".equals(decided.by())) {
                by = "guard:" + decided.index();
            } else if ("AFTER".equals(decided.by())) {
                by = "after:" + decided.rule() + ":" + decided.days();
            } else {
                by = "otherwise";
            }
            if (observance.extents().size() != 2) {
                throw new IllegalStateException("an observance has " + observance.extents().size() + " extents, not two");
            }
            FestivalExtent earlier = observance.extents().get(0);
            FestivalExtent later = observance.extents().get(1);
            put(prefix + "-" + k, String.join(" ",
                    observance.rule(),
                    observance.month().fullKey(),
                    lower(observance.adhika()),
                    observance.day().month() + "-" + observance.day().day(),
                    observance.caseHeld(),
                    by,
                    observance.choice(),
                    number(observance.tithi().fromJd()),
                    number(earlier.held()),
                    number(later.held())));
        }
        for (int k = 0; k < answer.ekadashis().size(); k += 1) {
            EkadashiFast fast = answer.ekadashis().get(k);
            put(prefix + "-ekadashi-" + k, String.join(" ",
                    fast.rule(),
                    fast.tithi().fullKey(),
                    fast.month().fullKey(),
                    lower(fast.adhika()),
                    fast.day().month() + "-" + fast.day().day(),
                    fast.piercedAt() == null || fast.piercedAt().isEmpty() ? "-" : fast.piercedAt(),
                    lower(fast.pierced()),
                    fast.excess(),
                    fast.choice(),
                    number(fast.tithis().get(1).fromJd())));
        }
    }

    /** An Ashta Koota as every runner prints it, under {@code prefix}. */
    private static void putAshta(String prefix, AshtaKoota matched) {
        put(prefix + "-matching", number(matched.total()));
        for (KootaRow koota : matched.kootas()) {
            put(prefix + "-matching-" + koota.reading().koota().fullKey(), number(koota.points()) + " "
                    + number(koota.maxPoints()) + " " + readingText(koota.reading()));
        }
    }

    /** Ten considerations as every runner prints them, under {@code prefix}. */
    private static void putPorutham(String prefix, Porutham ten) {
        PoruthamException clauses = ten.exception();
        put(prefix + "-porutham", ten.agreeing() + " " + ten.chiefAgreeing() + " " + flag(clauses.oneLord()) + " "
                + flag(clauses.lordsFriendly()) + " " + flag(clauses.opposite()));
        for (PoruthamRow consideration : ten.considerations()) {
            put(prefix + "-porutham-" + consideration.reading().koota().fullKey(), flag(consideration.agrees()) + " "
                    + flag(consideration.lifted()) + " " + poruthamText(consideration.reading()));
        }
    }

    /** A reduction's steps, as every runner writes them. */
    private static String steps(Reduction reduction) {
        return ints("/", reduction.steps());
    }

    /**
     * Two numerology requests, as every runner asks them: Balliett's own
     * example under the sources' readings, and her John Wanamaker under every
     * baseline reading.
     */
    private static void putNumerology(Context ctx) {
        Map<String, Object> baseline = map(
                "masters", "ELEVEN_TWENTY_TWO_THIRTY_THREE",
                "nameReduction", "WHOLE",
                "chaldeanCompound", "LETTER_TOTAL",
                "birthReduction", "DIGIT_SUM",
                "nonLatin", "SKIP");
        List<String> names = List.of("Henry Elder", "John Wanamaker");
        List<LocalDate> births = List.of(LocalDate.of(1872, 1, 17), LocalDate.of(1838, 7, 11));
        List<Map<String, Object>> rules = new ArrayList<>();
        rules.add(null);
        rules.add(baseline);
        for (int n = 0; n < names.size(); n += 1) {
            NumerologyProfile read = rules.get(n) == null
                    ? ctx.numerology().profile(names.get(n), births.get(n))
                    : ctx.numerology().profile(names.get(n), births.get(n), rules.get(n));
            Map<String, NameNumber> systems = new LinkedHashMap<>();
            systems.put("pythagorean", read.pythagoreanName());
            systems.put("chaldean", read.chaldeanName());
            for (Map.Entry<String, NameNumber> system : systems.entrySet()) {
                NameNumber named = system.getValue();
                String compound = named.compound() == null ? "NONE" : String.valueOf(named.compound());
                String at = "numerology-" + n + "-" + system.getKey();
                put(at, named.total() + " " + compound + " " + steps(named.reduction()));
                for (int w = 0; w < named.words().size(); w += 1) {
                    WordNumber word = named.words().get(w);
                    String letters = join(",", word.letters(), letter -> letter.letter() + letter.value());
                    put(at + "-word-" + w, word.text() + " " + letters + " " + word.total() + " " + steps(word.reduction()));
                }
            }
            BirthNumber birth = read.pythagoreanBirth();
            String summed = birth.sum() == null ? "NONE" : steps(birth.sum());
            String apart = orDash(ints(",", birth.apart()));
            put("numerology-" + n + "-birth", steps(birth.month()) + " " + steps(birth.day()) + " " + steps(birth.year())
                    + " " + summed + " " + apart);
            put("numerology-" + n + "-chaldean-birth",
                    steps(read.chaldeanBirth().birth()) + " " + steps(read.chaldeanBirth().year()));
            BaselineNumerology own = read.baseline();
            put("numerology-" + n + "-baseline", own == null
                    ? "NONE"
                    : steps(own.soul()) + " " + steps(own.personality()) + " " + steps(own.chaldeanDestiny()));
        }
    }

    /** A calendar date as {@code year-month-day}. */
    private static String ymd(CalendarDate day) {
        return day.year() + "-" + day.month() + "-" + day.day();
    }

    /**
     * The rashifal batch every runner sends: a week read at sunrise with the
     * baseline's weekly scores, and a day read at 06:00 reporting only Mars's
     * and Saturn's events.
     */
    private static void putRashifal(Context geo, Observer place) {
        RashifalRequest week = new RashifalRequest(date(2024, 6, 17), date(2024, 6, 23), place, 20700, null, null, null);
        RashifalRequest oneDay = new RashifalRequest(date(2024, 6, 17), null, place, 20700,
                map("at", "CLOCK", "hour", 6, "minute", 0), List.of(Graha.MARS, Graha.SATURN), null);
        List<RashifalAnswer> answers = geo.chart().rashifalMany(List.of(week, oneDay), "WEEKLY");
        for (int n = 0; n < answers.size(); n += 1) {
            RashifalAnswer answer = answers.get(n);
            RashifalPeriod period = answer.period();
            put("rashifal-" + n + "-period", ymd(period.first()) + " " + ymd(period.last()) + " "
                    + ymd(period.reference()) + " " + number(period.instant()));
            RashifalPanchanga limbs = period.panchanga();
            put("rashifal-" + n + "-panchanga",
                    limbs.tithi().fullKey() + " " + limbs.yoga().fullKey() + " " + limbs.muhurtaYogas());
            int transits = Math.min(period.transits().size(), period.retrograde().size());
            for (int g = 0; g < transits; g += 1) {
                Transit transit = period.transits().get(g);
                put("rashifal-" + n + "-transit-" + g, transit.sign().fullKey() + " " + number(transit.degrees()) + " "
                        + flag(period.retrograde().get(g)));
            }
            for (int r = 0; r < period.readings().size(); r += 1) {
                RashiReading reading = period.readings().get(r);
                SaturnStanding saturn = reading.saturn();
                String verdicts = join(",", reading.gochar().grahas(), g -> g.verdict().key());
                String sadeSati = saturn.sadeSati() == null || saturn.sadeSati().isEmpty() ? "-" : saturn.sadeSati();
                put("rashifal-" + n + "-" + r, reading.rashi().fullKey() + " " + saturn.house() + " " + sadeSati + " "
                        + flag(saturn.spell()) + " " + verdicts + " " + reading.events().size());
                for (int k = 0; k < reading.events().size(); k += 1) {
                    RashifalEvent event = reading.events().get(k);
                    Hit hit = event.hit();
                    put("rashifal-" + n + "-" + r + "-event-" + k, number(hit.instant()) + " " + hit.graha().fullKey()
                            + " " + hit.event().kind().key() + " " + event.sign().fullKey() + " " + event.house() + " "
                            + flag(event.goodHouse()));
                }
                if (answer.baseline() != null) {
                    BaselineScore score = answer.baseline().get(r);
                    String named = join(",", score.keyInfluences(),
                            k -> k.graha().fullKey() + ":" + k.house() + ":" + k.verdict().key());
                    LuckyElements lucky = score.lucky();
                    put("rashifal-" + n + "-" + r + "-baseline", score.overall() + " "
                            + join(",", score.areas(), area -> String.valueOf(area.getValue())) + " " + orNone(named)
                            + " " + lucky.colour() + " " + lucky.number() + " " + lucky.day().fullKey() + " "
                            + lucky.direction().fullKey());
                }
            }
        }
    }

    /**
     * Two pairs of names, as every runner asks them: a Devanagari pair whose
     * groom's syllable is Abhijit's, placed in Shravana, and an IAST pair.
     */
    private static void putNaam(Context ctx) {
        // The same code points as the Python runner's literals, escaped so no
        // editor can normalise them.
        List<String> brides = List.of("प्रिया", "kṛṣṇā");
        List<String> grooms = List.of("ज़ोया", "śyāma");
        List<Map<String, Object>> rules = List.of(
                map("name", map("abhijit", "SHRAVANA"), "koota", map("nadiDosha", "MIDDLE_ONLY")),
                map("name", map("latin", "IAST"), "porutham", map("deerghaBeyond", "SEVENTH")));
        for (int n = 0; n < brides.size(); n += 1) {
            NaamMilan read = ctx.matching().naam(brides.get(n), grooms.get(n), rules.get(n));
            Map<String, NameSyllable> sides = new LinkedHashMap<>();
            sides.put("bride", read.bride());
            sides.put("groom", read.groom());
            for (Map.Entry<String, NameSyllable> side : sides.entrySet()) {
                NameSyllable name = side.getValue();
                String star = name.nakshatra() == null ? "NONE" : name.nakshatra().fullKey();
                put("naam-" + n + "-" + side.getKey(), name.cell() + " " + star + " " + name.quarter() + " "
                        + name.varga().key());
            }
            VargaKoota varga = read.varga();
            put("naam-" + n + "-varga", varga.bride().key() + " " + varga.groom().key() + " " + varga.relation().key());
            putAshta("naam-" + n, read.ashta());
            putPorutham("naam-" + n, read.porutham());
        }
    }

    /**
     * A local day's every field, under the same keys for a chart's day and an
     * almanac's, because the two layers hand back one record.
     */
    private static void putDay(String prefix, LocalDay day) {
        put(prefix + "-vara", day.vara().fullKey());
        put(prefix + "-sunrise", day.sunrise());
        put(prefix + "-sunset", day.sunset());
        put(prefix + "-next-sunrise", day.nextSunrise());
        put(prefix + "-date", day.date().year() + "-" + day.date().month() + "-" + day.date().day());
        put(prefix + "-calendar", day.date().calendar().fullKey());
        put(prefix + "-era", day.date().era() == null ? "none" : day.date().era().fullKey());
        put(prefix + "-era-year", day.date().eraYear());
        put(prefix + "-resolution", day.date().resolution().key());
        put(prefix + "-polar", day.polar() == null ? "none" : day.polar().kind().key() + "/" + day.polar().policy().key());
        String convention;
        if (day.air() != null && day.convention() != null) {
            convention = day.convention().key() + " " + number(day.air().pressureHpa()) + " hPa "
                    + number(day.air().temperatureC()) + " C";
        } else if (day.convention() != null) {
            convention = day.convention().key();
        } else {
            double altitude = day.customAltitudeDeg() == null ? 0.0 : day.customAltitudeDeg();
            convention = "custom " + number(altitude);
        }
        put(prefix + "-convention", convention);
    }

    /** A rendered message's parts as every runner writes them, the one text part made where no markup came. */
    private static String partShape(IntlRender rendered) {
        String parts = rendered.parts();
        List<?> written = list(Json.read(parts == null || parts.isEmpty() ? "[]" : parts));
        if (written.isEmpty()) {
            // Python's `message_parts`: no markup is the one text part, made
            // rather than carried. The Java binding has no such helper, so the
            // runner makes it from the blob's `parts` as Python's does.
            return "text:" + rendered.text();
        }
        List<String> said = new ArrayList<>();
        for (Object raw : written) {
            if ("text".equals(dig(raw, "type"))) {
                said.add("text:" + py(dig(raw, "value")));
            } else {
                Map<String, String> options = new TreeMap<>(ParityRunner::byCodePoint);
                if (dig(raw, "options") instanceof Map<?, ?> fields) {
                    for (Map.Entry<?, ?> option : fields.entrySet()) {
                        options.put(String.valueOf(option.getKey()), py(option.getValue()));
                    }
                }
                said.add(py(dig(raw, "kind")) + ":" + py(dig(raw, "name")) + "("
                        + join(",", new ArrayList<>(options.entrySet()), o -> o.getKey() + "=" + o.getValue()) + ")");
            }
        }
        return String.join("|", said);
    }

    /** A Tajika saham as every runner prints it. */
    private static String sahamSaid(TajikaSaham p) {
        String axis = p.inNodeAxis() == null ? "null" : lower(p.inNodeAxis());
        String seven = join(" ", p.seven(), s -> s.drishti().key() + "/" + s.relation().key() + "/" + flag(s.company()));
        return String.join(" | ",
                fixed(p.longitudeDeg(), 6) + " " + p.sign().fullKey() + " " + p.lord().fullKey() + " " + p.house() + " "
                        + lower(p.addedSign()),
                "S:" + join(",", p.strong(), SahamStrong::key) + " W:" + join(",", p.weak(), SahamWeak::key),
                p.lordVishwa() + " " + p.lordHarsha().key() + " " + axis,
                seven);
    }

    /** The dignities an essential dignity holds, in its fields' order, or {@code -}. */
    private static String dignitiesHeld(EssentialDignity dignity) {
        return orDash(String.join(",", dignityFlags(dignity)));
    }

    /** The dignity flags held, in Python's {@code dignity_flags} order. */
    private static List<String> dignityFlags(EssentialDignity d) {
        List<String> held = new ArrayList<>();
        if (d.house()) {
            held.add("house");
        }
        if (d.exaltation()) {
            held.add("exaltation");
        }
        if (d.triplicity()) {
            held.add("triplicity");
        }
        if (d.term()) {
            held.add("term");
        }
        if (d.face()) {
            held.add("face");
        }
        if (d.detriment()) {
            held.add("detriment");
        }
        if (d.fall()) {
            held.add("fall");
        }
        return held;
    }

    /** An almuten's totals, its winners and its partakers. */
    private static String ranked(Almuten almuten) {
        String totals = join(",", almuten.totals(), at -> String.valueOf(at.total()));
        return totals + " " + grahaKeys(almuten.almutens()) + " " + grahaKeys(almuten.partakers());
    }

    /** A perfection ahead, or {@code -}. */
    private static String perfectionSaid(Perfection found) {
        if (found == null) {
            return "-";
        }
        return found.planet().fullKey() + " " + found.aspect().key() + " " + number(found.days()) + " "
                + number(found.gapDeg());
    }

    /** A KP level: its lord and span. */
    private static String level(KpLevel at) {
        return at.lord().fullKey() + " " + at.span().start() + " " + at.span().end();
    }

    /** A KP point's lords. */
    private static String lords(KpLords of) {
        return of.sign().fullKey() + " " + level(of.star()) + " " + level(of.sub()) + " " + level(of.subSub());
    }

    /** A ruler's rejection, or {@code -}. */
    private static String rejection(KpRejection by) {
        return by == null ? "-" : by.retrograde().fullKey() + ":" + lower(by.byStar());
    }

    /** A Julian day that may be absent. */
    private static String bound(Double jd) {
        return jd == null ? "-" : number(jd.doubleValue());
    }

    /** An eclipse's moment, or {@code -}. */
    private static String eclipseMoment(EclipseMoment m) {
        return m == null ? "-" : number(m.at()) + "@" + number(m.altitudeDeg());
    }

    /** An eclipse seen from here, or {@code -}. */
    private static String eclipseSeen(EclipseSeen stretch) {
        return stretch == null ? "-" : number(stretch.from()) + ".." + number(stretch.to());
    }

    /** A segment of an outline by its kind, as Python names its class less {@code Segment}. */
    private static String segmentName(Segment step) {
        return step.getClass().getSimpleName().replaceFirst("Segment$", "").toUpperCase(Locale.ROOT);
    }

    /** A surface line: the canonical {@code area.operation} path, and this binding's member beside it. */
    private static void surface(String path, Reference member) {
        put("surface." + path, member == null ? "missing" : "present");
    }

    // ── The scenario ──────────────────────────────────────────────────────

    private static void run() {
        Teistro teistro = Teistro.open();

        // ── The library itself ────────────────────────────────────────────
        put("abi", teistro.abiVersion());
        put("sdk", teistro.sdkVersion());
        put("catalogue-version", teistro.catalogueVersion());
        put("default-profile", teistro.defaultProfile());
        Map<String, Object> build = teistro.buildInfo();
        put("build-sdk", text(build.get("sdk")));
        put("build-abi", whole(build.get("abi")));
        put("build-catalogue", whole(build.get("catalogue")));
        put("build-commit", text(build.get("commit")));
        put("build-dirty", Boolean.TRUE.equals(build.get("dirty")));
        put("build-target", text(build.get("target")));

        // ── A context ─────────────────────────────────────────────────────
        try (Context ctx = teistro.context(ContextOptions.builder()
                .profile("nepali-default").locale("ne-Deva-NP").testProvider(true).build())) {
            put("profile", ctx.profile());
            put("locale", ctx.intl().locale());
            put("settings-hash", ctx.settingsHash());
            put("settings-fnv", fnv(ctx.settingsJson()));
            putNaam(ctx);
            putNumerology(ctx);

            // ── The calendars ─────────────────────────────────────────────
            CalendarDate day = date(2015, 4, 14);
            CalendarDate bs = ctx.calendar().convert(day, Calendar.BIKRAM_SAMBAT);
            put("bs-year", bs.year());
            put("bs-month", bs.month());
            put("bs-day", bs.day());
            put("bs-era", bs.era() == null ? null : bs.era().fullKey());
            put("bs-era-year", bs.eraYear());
            put("bs-resolution", bs.resolution().key());
            long fixed = ctx.calendar().fixedOf(day);
            put("fixed", fixed);
            put("weekday", ctx.calendar().weekdayOf(day));
            put("month-length", ctx.calendar().monthLength(Calendar.GREGORIAN, 2024, 2));
            put("is-leap", ctx.calendar().isLeap(Calendar.GREGORIAN, 2024));
            put("jd-of-fixed", teistro.julianDayOfFixed(fixed));
            CalendarFixedOfJdResult back = teistro.fixedOfJulianDay(2457126.75);
            put("fixed-of-jd", back.value());
            put("fraction-of-jd", back.fraction());

            // ── Time ──────────────────────────────────────────────────────
            CivilDateTime civil = new CivilDateTime(date(1986, 1, 1), new CivilTime(0, 20, 0, true, 0));
            ZoneSpec zone = new ZoneSpec(ZoneKind.IANA, 0, new Longitude(0), "Asia/Kathmandu");
            ZoneResolution resolved = ctx.time().resolve(civil, zone);
            put("resolve-jd", resolved.instantJdUtc());
            put("resolve-offset", resolved.offsetSeconds());
            put("resolve-era", resolved.era().key());
            put("resolve-source", resolved.source().key());
            put("resolve-time-known", resolved.timeKnown());
            put("resolve-tzdb", resolved.tzdbVersion());
            put("resolve-warnings", resolved.warnings().size());
            TimeCivilResult civilBack = ctx.time().civilOf(resolved.instantJdUtc(), zone, Calendar.GREGORIAN);
            put("civil-year", civilBack.civil().date().year());
            put("civil-minute", civilBack.civil().time().minute());
            put("civil-offset", civilBack.resolution().offsetSeconds());
            TimeConversion tt = ctx.time().convert(2451544.5, Scale.UTC, Scale.TT);
            put("tt-jd", tt.jd());
            put("tt-delta-t", tt.deltaTSeconds());
            put("tt-delta-t-source", tt.deltaTSource().key());
            put("tt-delta-t-model", tt.deltaTModel());
            DeltaT delta = ctx.time().deltaT(2451544.5);
            put("delta-t-seconds", delta.seconds());
            put("delta-t-source", delta.source().key());

            // ── Keys ──────────────────────────────────────────────────────
            long identifier = ctx.keys().id("graha.SUN");
            put("key-id", identifier);
            put("key-name", ctx.keys().name(identifier));
            try {
                ctx.keys().id("graha.SUNN");
                put("refusal", "none");
            } catch (TeistroException error) {
                put("refusal-status", error.status().key());
                put("refusal-detail", error.detail());
                put("refusal-hint-names-sun", error.hint().contains("SUN"));
            }

            // ── The locale engine ─────────────────────────────────────────
            IntlRender rendered = ctx.intl().render("sdk.reason.grahaInBhava",
                    map("graha", map("$entity", "graha.JUPITER"), "bhava", 7));
            put("render-fnv", fnv(rendered.text()));
            put("render-length", rendered.text().codePointCount(0, rendered.text().length()));
            put("render-resolved-from", rendered.resolvedFrom());
            put("render-fallback", rendered.isFallback() != 0);
            // A rendered message's parts, which is what a rich renderer walks.
            // `sdk.reason.lordship` is one of the two shipped messages carrying
            // `{#b}`; the plain one beside it holds every binding to the rule
            // that no markup means the one text part, made rather than carried.
            IntlRender rich = ctx.intl().render("sdk.reason.lordship",
                    map("graha", map("$entity", "graha.JUPITER"), "bhava", 5));
            put("render-rich-parts", partShape(rich));
            put("render-plain-parts", partShape(rendered));
            put("has-message", ctx.intl().has("sdk.reason.grahaInBhava"));
            put("has-missing-message", ctx.intl().has("sdk.nope.missing"));
            put("transliterated", ctx.intl().transliterate(
                    "सूर्य बृहस्पति"));
            Messages.EntityForms sun = ctx.intl().entity("graha.SUN");
            put("entity-sun-name", sun.name());
            put("entity-sun-iast", sun.iast());
            put("entity-sun-glyph", sun.glyph());
            put("entity-sun-gender", sun.gender() == null ? null : sun.gender().value());
            put("message-graha-in-bhava",
                    ctx.intl().messages().sdk().reason().grahaInBhava(7, Messages.GrahaKey.JUPITER));
            put("message-bs-date", ctx.intl().messages().sdk().calendar().bikramSambat().date().long_(
                    1, "\u092c\u0948\u0936\u093e\u0916", 2072));

            // ── Positions ─────────────────────────────────────────────────
            Frame frame = teistro.canonicalFrame();
            put("frame-centre", frame.centre().key());
            put("frame-coordinates", frame.coordinates().key());
            put("frame-bits", teistro.packFrame(frame));
            put("frame-round-trip", teistro.unpackFrame(teistro.packFrame(frame)).centre() == frame.centre());
            PositionGrid sky = ctx.positions(new double[] {2451545.0, 2451546.0},
                    List.of(Body.SUN, Body.MOON, Body.MARS));
            put("cells", sky.cellCount());
            put("positions-scale", sky.timeScale().key());
            put("positions-bodies", join(",", sky.bodyKeys(), Body::key));
            for (int index = 0; index < sky.cellCount(); index += 1) {
                Cell cell = sky.at(index / sky.bodyCount(), index % sky.bodyCount());
                put("cell-" + index + "-lon", cell.longitude());
                put("cell-" + index + "-lat", cell.latitude());
                put("cell-" + index + "-dist", cell.distance());
                put("cell-" + index + "-lon-speed", cell.longitudeSpeed());
                put("cell-" + index + "-status", cell.status());
            }
            put("steps", join(",", sky.stepsApplied(), step -> step.name() + ":" + step.implementation().key()));
            put("provenance-fnv", fnv(sky.provenanceJson()));
            Provenance provenance = sky.provenance();
            put("provenance-profile", provenance.profile());
            put("provenance-settings-hash", provenance.settingsHash());
            put("provenance-provider-frame", provenance.provider().frame());

            // ── The chart the topocentric profile founds ──────────────────
            // `nepali-default`'s frame is topocentric, as every recorded chart
            // in the corpus is; the chart itself is compared.
            Observer place = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
            Chart placed = ctx.chart().found(2451545, place, 20700, ChartOptions.none());
            put("chart-under-topocentric", "founded");
            put("topocentric-steps", String.join(",", placed.batch().stepsApplied()));
            put("topocentric-lagna", placed.lagnaDeg());
            for (int j = 0; j < placed.grahas().size(); j += 1) {
                PlacedGraha graha = placed.grahas().get(j);
                put("topocentric-graha-" + j, graha.graha().fullKey());
                put("topocentric-graha-" + j + "-lon", graha.longitudeDeg());
                put("topocentric-graha-" + j + "-lat", graha.latitudeDeg());
                put("topocentric-graha-" + j + "-speed", graha.speedDegPerDay());
            }

            // ── A chart founded on a classical astronomy ──────────────────
            // The Surya Siddhanta by name, which every binding reaches
            // through the selector and must read back alike, deviation and all.
            try (Context classical = teistro.context(ContextOptions.builder()
                    .profile("surya-siddhanta").ephemeris(Ephemeris.SURYA_SIDDHANTA).build())) {
                Chart siddhanta = classical.chart().found(2447995.4895833335, place, 20700, ChartOptions.none());
                put("classical-steps", String.join(",", siddhanta.batch().stepsApplied()));
                put("classical-lagna", siddhanta.lagnaDeg());
                put("classical-sunrise", siddhanta.day().sunrise());
                Deviation deviation = Objects.requireNonNull(siddhanta.provenance().deviation(),
                        "the Surya Siddhanta chart carries no deviation");
                put("classical-deviation", deviation.model() + ": " + deviation.detail());
                for (int j = 0; j < siddhanta.grahas().size(); j += 1) {
                    put("classical-graha-" + j + "-lon", siddhanta.grahas().get(j).longitudeDeg());
                }
            }

            // ── A chart and an almanac, under a geocentric profile ────────
            // A layout of the consumer's own, registered on the context the
            // charts are drawn under: the South Indian row renamed.
            Map<String, Object> kerala;
            try (Context shipped = teistro.context(ContextOptions.builder().testProvider(true).build())) {
                kerala = shipped.chart().layout(ChartLayout.SOUTH_INDIAN);
            }
            kerala.put("key", "ACME_KERALA");
            // A dasha system of the consumer's own, the same definition every
            // runner registers (`03-design/dasha-kernels.md`).
            String parityDasha = "{\"kernel\":\"UDU\",\"key\":\"ACME_PARITY\",\"sources\":[\"the parity scenario\"],"
                    + "\"lords\":[{\"graha\":\"SUN\",\"years\":5},{\"graha\":\"MOON\",\"years\":10},"
                    + "{\"graha\":\"MARS\",\"years\":7},{\"graha\":\"MERCURY\",\"years\":12}],"
                    + "\"reference\":\"MULA\",\"count\":\"TO_REFERENCE\",\"span\":2,\"offset\":1,\"repeats\":true,"
                    + "\"year_length\":\"SAVANA_360\",\"depth\":2}";
            try (Context geo = teistro.context(ContextOptions.builder()
                    .profile("parashari-classical").locale("ne-Deva-NP").testProvider(true)
                    .layoutsJson(Json.write(List.of(kerala))).dashasJson("[" + parityDasha + "]").build())) {
                put("geo-profile", geo.profile());
                put("geo-settings-hash", geo.settingsHash());
                charts(ctx, geo, place);
                almanac(geo, place);
                putRashifal(geo, place);
            }

            // ── The eclipses ──────────────────────────────────────────────
            // September 2025 at Kathmandu over the built-in sky: a total lunar
            // eclipse seen whole and a partial solar one the place does not see.
            Eclipses eclipses;
            try (Context builtin = teistro.context(ContextOptions.builder()
                    .profile("nepali-default").ephemeris(Ephemeris.BUILTIN).build())) {
                eclipses = builtin.almanac().of(date(2025, 9, 1), date(2025, 9, 30), place, 20700,
                        null, null, false, true, false).eclipses().orElseThrow();
            }
            put("eclipses-hash", eclipses.provenance().contentHash());
            put("eclipses-count", eclipses.value().lunar().size() + " " + eclipses.value().solar().size());
            for (int k = 0; k < eclipses.value().lunar().size(); k += 1) {
                LunarEclipseHere lunarEclipse = eclipses.value().lunar().get(k);
                LunarEclipse found = lunarEclipse.eclipse();
                LunarEclipseView view = lunarEclipse.here();
                LunarContacts contacts = found.contacts();
                put("eclipses-lunar-" + k, String.join(" ", found.kind().fullKey(), found.shadow(),
                        number(found.greatest()), number(found.gamma()), number(found.umbralMagnitude()),
                        number(found.penumbralMagnitude())));
                put("eclipses-lunar-" + k + "-contacts", String.join(" ", bound(contacts.p1()), bound(contacts.u1()),
                        bound(contacts.u2()), bound(contacts.u3()), bound(contacts.u4()), bound(contacts.p4())));
                put("eclipses-lunar-" + k + "-here", String.join(" ", eclipseMoment(view.p1()),
                        eclipseMoment(view.u1()), eclipseMoment(view.u2()), eclipseMoment(view.greatest()),
                        eclipseMoment(view.u3()), eclipseMoment(view.u4()), eclipseMoment(view.p4()),
                        eclipseSeen(view.seen()), eclipseSeen(view.umbralSeen())));
            }
            for (int k = 0; k < eclipses.value().solar().size(); k += 1) {
                SolarEclipseHere solarEclipse = eclipses.value().solar().get(k);
                SolarEclipse found = solarEclipse.eclipse();
                SolarEclipseView view = solarEclipse.here();
                put("eclipses-solar-" + k, String.join(" ", found.kind().fullKey(), number(found.greatest()),
                        number(found.gamma()), number(found.magnitude()), number(found.point().latitude()),
                        number(found.point().longitude())));
                put("eclipses-solar-" + k + "-here", view == null
                        ? "-"
                        : String.join(" ", view.kind().fullKey(), number(view.magnitude()), number(view.obscuration()),
                                eclipseMoment(view.first()), eclipseMoment(view.second()),
                                eclipseMoment(view.third()), eclipseMoment(view.fourth()),
                                eclipseMoment(view.maximum()), eclipseSeen(view.seen())));
            }

            // ── The surface's shape ───────────────────────────────────────
            // These compare where an operation LIVES: every key is the
            // canonical `area.operation` path, and the member beside it is
            // this binding's own spelling of it, referenced in a lambda that
            // is never run so that javac checks it exists.
            surface("calendar.date_of", () -> ctx.calendar().dateOf(Calendar.GREGORIAN, 1));
            surface("calendar.fixed_of", () -> ctx.calendar().fixedOf(day));
            surface("calendar.convert", () -> ctx.calendar().convert(day, Calendar.BIKRAM_SAMBAT));
            surface("calendar.weekday_of", () -> ctx.calendar().weekdayOf(day));
            surface("calendar.month_length", () -> ctx.calendar().monthLength(Calendar.GREGORIAN, 2024, 2));
            surface("calendar.is_leap", () -> ctx.calendar().isLeap(Calendar.GREGORIAN, 2024));
            surface("time.resolve", () -> ctx.time().resolve(civil, zone));
            surface("time.civil_of", () -> ctx.time().civilOf(0, zone, Calendar.GREGORIAN));
            surface("time.convert", () -> ctx.time().convert(0, Scale.UTC, Scale.TT));
            surface("time.delta_t", () -> ctx.time().deltaT(0));
            surface("intl.locale", () -> ctx.intl().locale());
            surface("intl.render", () -> ctx.intl().render("", Map.of()));
            surface("intl.has", () -> ctx.intl().has(""));
            surface("intl.transliterate", () -> ctx.intl().transliterate(""));
            surface("intl.entity", () -> ctx.intl().entity(""));
            surface("intl.messages", () -> ctx.intl().messages());
            surface("intl.load_pack", () -> ctx.intl().loadPack(new byte[0]));
            surface("keys.id", () -> ctx.keys().id(""));
            surface("keys.name", () -> ctx.keys().name(0));
            surface("matching.naam", () -> ctx.matching().naam("", ""));
            surface("numerology.profile", () -> ctx.numerology().profile("", LocalDate.of(2000, 1, 1)));
            surface("frame.canonical", () -> ctx.frame().canonical());
            surface("frame.pack", () -> ctx.frame().pack(frame));
            surface("frame.unpack", () -> ctx.frame().unpack(0));
            surface("chart.layout", () -> ctx.chart().layout(""));
            surface("chart.found", () -> ctx.chart().found(0, place, 0, ChartOptions.none()));
            surface("chart.found_many", () -> ctx.chart().foundMany(new double[] {0}, place, 0, ChartOptions.none()));
            surface("almanac.of", () -> ctx.almanac().of(day, day, place, 0));
            surface("almanac.day", () -> ctx.almanac().day(day, place, 0));
            surface("engine.names", () -> ctx.ephemeris().names());
            surface("engine.signature", () -> ctx.ephemeris().signature(""));
            surface("engine.call", () -> ctx.ephemeris().call("", Map.of()));
            surface("engine.call_json", () -> ctx.ephemeris().callJson("", "{}"));
            surface("engine.manifest", () -> ctx.ephemeris().manifest());
            surface("engine.manifest_json", () -> ctx.ephemeris().manifestJson());
            surface("(root).engine", () -> ctx.ephemeris());
            surface("(root).positions", () -> ctx.positions(new double[] {0}, List.of(Body.SUN)));
            surface("(root).profile", () -> ctx.profile());
            surface("(root).settings", () -> ctx.settings());
            surface("(root).settings_json", () -> ctx.settingsJson());
            surface("(root).settings_hash", () -> ctx.settingsHash());
            surface("(root).dispose", ctx::close);
        }
    }

    /** The chart request every runner sends: every section, every reading. */
    private static ChartOptions everything() {
        List<Object[]> moieties = List.of(
                new Object[] {"SUN", 17}, new Object[] {"MOON", 12.5}, new Object[] {"MERCURY", 7},
                new Object[] {"VENUS", 8}, new Object[] {"MARS", 7.5}, new Object[] {"JUPITER", 12},
                new Object[] {"SATURN", 10}, new Object[] {"URANUS", 5}, new Object[] {"NEPTUNE", 5},
                new Object[] {"PLUTO", 5});
        List<Object> orbs = new ArrayList<>();
        for (Object[] orb : moieties) {
            orbs.add(map("graha", orb[0], "orbDeg", orb[1]));
        }
        return ChartOptions.builder()
                .vargas(Varga.D9, Varga.D10)
                .dashas(DashaSystem.VIMSHOTTARI, DashaSystem.CHARA, DashaSystem.KALACHAKRA,
                        DashaSystem.RELEASING_FORTUNE, DashaSystem.PROFECTION, DashaSystem.FIRDARIA,
                        DashaSystem.DECENNIALS, "dasha_system.ACME_PARITY")
                .drawing(ChartLayout.NORTH_INDIAN, Varga.D1)
                .drawing(ChartLayout.SOUTH_INDIAN, Varga.D9)
                .drawing(ChartLayout.WESTERN_WHEEL, Varga.D1)
                .drawing("chart_layout.ACME_KERALA", Varga.D9)
                // Python's `theme="DARK"` crosses as `{"extends": "DARK"}`.
                .theme(map("extends", "DARK"))
                .rules(map("shipped", List.of("NABHASAS"), "longevity", true,
                        "ayurdaya", map("enemy_exempt", "mars", "enmity", "compound", "rising", "every")))
                .interpret(map("placements", true, "readings", true, "strength", true, "houses", true,
                        "positions", true, "aspects", true, "conditions", true, "karakas", true))
                .aspects(true)
                .points(true)
                .houses(true)
                .ashtakavarga(true)
                .vimshopaka(true)
                .vaiseshikamsa(true)
                .dashaPhala(true)
                .jaimini(true)
                .avakahada(true)
                .outerPlanets(true)
                .gochar(map("instants", List.of(2460676.5, 2460736.5), "ashtakavarga", true))
                .hits(map("from", 2460676.5, "to", 2460736.5, "grahas", List.of("SUN", "MERCURY", "SATURN"),
                        "aspects", List.of(0, 90, 180), "orbDeg", 2))
                .sadeSati(map("from", 2460676.5, "to", 2464329.0, "reckoning", "DEGREE", "spells", List.of(4, 7, 8)))
                .kp(map("number", 74, "anyAyanamsha", true))
                .fortitudes(map(
                        "dignities", map("sectRule", "DAYLIGHT",
                                "rules", map("terms", "EGYPTIAN", "triplicities", "PTOLEMY"),
                                "scores", map("peregrine", 0)),
                        "rules", map("beamsDeg", 15, "combustionInSign", false,
                                "partile", map("WITHIN", map("orbDeg", 1)),
                                "siege", map("WITHIN", map("spanDeg", 30))),
                        "scores", map("regulus", 5),
                        "almuten", map("fortune", "REVERSED_BY_NIGHT")))
                .lots(map("fortune", "REVERSED_WHILE_MOON_UP"))
                .considerations(map("moonLateFromDeg", 25))
                .perfection(map("house", 7, "rules", map("horizonDays", 120)))
                .prashna(map("question", map("house", 7, "number", 14), "rules", map("score", "BASELINE")))
                .remedies(map("at", 2460676.5, "rules", map("shanti", map("rik", "YAJNAVALKYA"))))
                .westernAspects(map(
                        "aspects", List.of("CONJUNCTION", "SEXTILE", "SQUARE", "TRINE", "QUINCUNX", "OPPOSITION"),
                        "orbs", map("model", "MOIETIES", "orbs", orbs)))
                .parallels(map("orbDeg", 1.5))
                .antiscia(map("cusps", map()))
                .midpoints(map("orbDeg", 1.5))
                .westernHouses(map())
                .harmonic(map("number", 5))
                .matching(map("partner", partner(), "partnerRole", "BRIDE", "rules", map("bhakootLift", "GARGA"),
                        "porutham", map("lordsFriendship", "ONE_WAY"),
                        "kuja", map("houses", "WITH_SECOND", "from", "LAGNA_MOON_VENUS")))
                .synastry(map("partner", partner(), "aspects", List.of("CONJUNCTION", "SQUARE", "TRINE", "OPPOSITION"),
                        "zodiac", "CHARTS", "parallels", map("orbDeg", 1.5), "antiscia", map("orbs", map("model", "LEO")),
                        "midpoints", map("orbDeg", 1.5), "composite", true, "davison", true))
                .progressions(map("at", 2470000.5, "year", "NOON_SIDEREAL_TIME", "angles", "SOLAR_ARC_LONGITUDE",
                        "direction", "NAIBOD", "contacts", map("from", 2462000.5, "to", 2465652.5,
                                "grahas", List.of("MOON", "SUN"), "points", List.of("LAGNA", "MARS", "VENUS"),
                                "aspects", List.of(0, 45, 90, 135, 180))))
                .shadbala(true)
                .bhavaBala(true)
                .state(true)
                .build();
    }

    /**
     * The partner every runner matches against, as the boundary reads it:
     * Python's {@code _partner_wire} writes the observer as {@code place} and
     * the clock as {@code utcOffsetSeconds}.
     */
    private static Map<String, Object> partner() {
        return map("instant", 2451545.25,
                "place", map("latitude", -33.87, "longitude", 151.21, "altitude", 0.0),
                "utcOffsetSeconds", 36000);
    }

    /** Every chart section, over two instants and the annual charts under all three readings. */
    private static void charts(Context ctx, Context geo, Observer place) {
        // Two instants, so a per-chart section that ran charts-outermost the
        // wrong way round shows up as the second chart's values in the
        // first's place; two divisional charts, so a transposed stride shows.
        ChartBatch charts = geo.chart().foundMany(new double[] {2460482.5, 2460600.25}, place, 20700, everything());
        put("chart-varga-count", charts.decoded().vargaCount());
        put("chart-drishti-table", charts.decoded().drishtiTable());
        put("chart-count", charts.size());
        put("chart-kind", charts.kind().fullKey());
        put("chart-place-lat", charts.place().latitudeDeg().value());
        put("chart-place-lon", charts.place().longitudeDeg().value());
        put("chart-model-fnv", fnv(charts.model()));
        put("chart-steps", String.join(",", charts.stepsApplied()));
        put("chart-provenance-fnv", fnv(charts.provenanceJson()));
        put("chart-provenance-profile", charts.provenance().profile());
        put("chart-graha-count", charts.decoded().grahaCount());

        for (Chart chart : charts) {
            chart(ctx, chart);
        }

        // The annual charts, under all three readings, each asking the
        // sixteen yogas, the sahams and the annual dashas a different way.
        for (String reading : List.of("SIDEREAL", "TROPICAL", "MEAN")) {
            Map<String, Object> varsha = map("reading", reading, "through", 12, "place", "birth");
            if (!reading.equals("MEAN")) {
                varsha.put("matters", "all");
                varsha.put("sahams", "all");
                varsha.put("dashas", "all");
            }
            if (reading.equals("TROPICAL")) {
                // Python writes these rule records in the boundary's casing
                // (`_varsha_json`): `saham_rules` as `sahamRules`, its keys
                // camel-cased, and `dasha_rules` likewise.
                varsha.put("yogas", map("tambira", "EITHER_LORD"));
                varsha.put("sahamRules", map("addSign", "SIGNS", "houses", "EQUAL", "roga", "SATURN"));
                varsha.put("dashaRules", map("clock", "EVEN", "balance", "ENTRY_MOON", "birthPeriod", "ELAPSED",
                        "depth", 3));
            }
            ChartBatch years = geo.chart().foundMany(new double[] {2460482.5, 2460600.25}, place, 20700,
                    ChartOptions.builder().varsha(varsha).build());
            for (int i = 0; i < years.size(); i += 1) {
                varsha(years.get(i), i, reading);
            }
        }

        Chart single = geo.chart().found(2460482.5, place, 20700, ChartOptions.none());
        put("chart-single-lagna", single.lagnaDeg());
        put("chart-single-agrees", single.lagnaDeg() == charts.at(0).lagnaDeg());
    }

    /** One chart's every section, under {@code chart-<index>}. */
    private static void chart(Context ctx, Chart chart) {
        int i = chart.index();
        String c = "chart-" + i;
        put(c + "-content-hash", chart.provenance().contentHash());
        put(c + "-instant", chart.instant());
        put(c + "-lagna", chart.lagnaDeg());
        put(c + "-day-lagna", chart.dayLagnaDeg());
        put(c + "-ayanamsha", chart.ayanamshaOffsetDeg());
        put(c + "-day-part", chart.dayPart().key());
        put(c + "-day-elapsed", chart.dayElapsed());
        putDay(c, chart.day());
        ChartTiming timing = chart.timing();
        put(c + "-ghati", timing.ghati());
        put(c + "-pala", timing.pala());
        put(c + "-vipala", timing.vipala());
        put(c + "-hora-number", timing.horaNumber());
        put(c + "-hora-lord", timing.horaLord().fullKey());
        for (int j = 0; j < chart.states().size(); j += 1) {
            GrahaState state = chart.states().get(j);
            String key = c + "-state-" + j;
            put(key, state.graha().fullKey());
            put(key + "-sign", state.sign().fullKey());
            put(key + "-house", state.house());
            put(key + "-dignity", state.dignity().fullKey());
            put(key + "-natural", state.friendship().natural().fullKey());
            put(key + "-compound", state.friendship().compound().fullKey());
            put(key + "-dispositor",
                    state.friendship().dispositor() != null ? state.friendship().dispositor().fullKey() : "none");
            put(key + "-burning", state.combustion().burning().key());
            Double fromSun = state.combustion().fromSunDeg();
            put(key + "-from-sun", fromSun != null ? fromSun : "none");
            Double orb = state.combustion().orbDeg();
            put(key + "-orb", orb != null ? orb : "none");
            put(key + "-age", state.age().fullKey());
            put(key + "-wakefulness", state.wakefulness().fullKey());
            put(key + "-deeptadi", state.deeptadi() != null ? state.deeptadi().fullKey() : "none");
            put(key + "-sayanadi", state.sayanadi() != null
                    ? state.sayanadi().avastha().fullKey() + " "
                            + join(",", state.sayanadi().cheshtas(), AvasthaCheshta::fullKey)
                    : "none");
            put(key + "-holding", orNone(join(",", state.lajjitadi().holding(), AvasthaLajjitadi::fullKey)));
            put(key + "-undecided", orNone(join(",", state.lajjitadi().undecided(), AvasthaLajjitadi::fullKey)));
            put(key + "-war", state.war() != null
                    ? state.war().opponent().fullKey() + ":" + lower(state.war().isWinner())
                    : "none");
            put(key + "-sign-edge", state.boundaries().signDeg());
        }
        for (ServiceBhava read : chart.bhavas()) {
            put(c + "-bhava-" + read.number() + "-sign", read.sign().fullKey());
            put(c + "-bhava-" + read.number() + "-lord", read.lord().fullKey());
            put(c + "-bhava-" + read.number() + "-quadrant", read.quadrant().key());
        }
        List<DerivedPoint> found = chart.points();
        put(c + "-point-count", found.size());
        for (int k = 0; k < found.size(); k += 1) {
            DerivedPoint derived = found.get(k);
            put(c + "-point-" + k, derived.point().fullKey());
            put(c + "-point-" + k + "-lon", derived.longitudeDeg());
            put(c + "-point-" + k + "-sign", derived.sign().fullKey());
            put(c + "-point-" + k + "-sign-edge", derived.boundaries().signDeg());
        }
        // Every drishti, because the count differs from chart to chart.
        List<Drishti> drishti = chart.aspects();
        put(c + "-aspect-count", drishti.size());
        for (int k = 0; k < drishti.size(); k += 1) {
            Drishti one = drishti.get(k);
            put(c + "-aspect-" + k, one.fromGraha().fullKey() + ">" + one.to().fullKey());
            put(c + "-aspect-" + k + "-houses", one.houses());
            put(c + "-aspect-" + k + "-strength", one.strength().key());
            put(c + "-aspect-" + k + "-from-sign", one.fromEdge().signDeg());
            put(c + "-aspect-" + k + "-to-sign", one.toEdge().signDeg());
        }
        Map<String, Object> answered = chart.rules().orElseThrow();
        put(c + "-rules-present", join(",", list(answered.get("present")), held -> String.valueOf(dig(held, "rule"))));
        put(c + "-rules-pindayu", dig(answered, "longevity", "ayurdaya", "pindayu", "years"));
        put(c + "-rules-jeevasarman", dig(answered, "longevity", "ayurdaya", "jeevasarman", "years"));
        put(c + "-rules-rays", dig(answered, "longevity", "rasmi", "total"));
        Object span = dig(answered, "longevity", "choice", "ayus");
        put(c + "-rules-span", truthy(span) ? span : "");
        Object dasayus = dig(answered, "longevity", "dasayus");
        put(c + "-rules-dasayus", truthy(dasayus) ? dig(dasayus, "years") : (Object) 0.0);
        put(c + "-rules-chakrayus", dig(answered, "longevity", "chakrayus", "years"));
        Object spans = dig(answered, "longevity", "ashtakavarga");
        put(c + "-rules-bhinna", truthy(spans) ? dig(spans, "bhinna") : (Object) 0.0);
        put(c + "-rules-samudaya", truthy(spans) ? dig(spans, "samudaya") : (Object) 0.0);
        put(c + "-rules-occupied", truthy(spans) ? dig(spans, "occupied") : (Object) 0.0);
        // Every item said, not merely counted, over whatever the request
        // asked for rather than a list written here.
        Map<String, Object> plans = chart.plans().orElseThrow();
        for (Map.Entry<String, Object> composed : plans.entrySet()) {
            List<?> items = list(composed.getValue());
            put(c + "-plan-" + composed.getKey() + "-count", items.size());
            for (int n = 0; n < items.size(); n += 1) {
                Object item = items.get(n);
                String key = String.valueOf(dig(item, "key"));
                Object params = dig(item, "params");
                String said = ctx.intl().renderJson(key, Json.write(params == null ? Map.of() : params)).text();
                put(c + "-plan-" + composed.getKey() + "-" + n, key + ": " + said);
            }
        }
        for (int d = 0; d < chart.drawings().size(); d += 1) {
            Drawing drawing = chart.drawings().get(d);
            String key = c + "-drawing-" + d;
            put(key, drawing.layoutKey());
            put(key + "-varga", drawing.varga().fullKey());
            put(key + "-cells", drawing.cells().size());
            put(key + "-frames", drawing.frame().size());
            put(key + "-marks", drawing.marks().size());
            put(key + "-svg", drawing.svg());
            for (int n = 0; n < drawing.cells().size(); n += 1) {
                DrawnCell drawn = drawing.cells().get(n);
                String where = key + "-cell-" + n;
                put(where + "-sign", drawn.sign().fullKey());
                put(where + "-house", drawn.house());
                put(where + "-lagna", drawn.lagna());
                put(where + "-ring", drawn.ring());
                put(where + "-bodies", orNone(String.join(",", drawn.bodies())));
                put(where + "-label", number(drawn.label().x()) + "," + number(drawn.label().y()));
                put(where + "-anchor", number(drawn.anchor().x()) + "," + number(drawn.anchor().y()));
                put(where + "-start", number(drawn.outline().start().x()) + "," + number(drawn.outline().start().y()));
                put(where + "-steps", join(",", drawn.outline().segments(), ParityRunner::segmentName));
            }
            for (int m = 0; m < drawing.marks().size(); m += 1) {
                DrawnMark mark = drawing.marks().get(m);
                String where = key + "-mark-" + m;
                put(where, mark.body());
                put(where + "-at", number(mark.at().x()) + "," + number(mark.at().y()));
                put(where + "-lon", mark.longitudeDeg());
            }
        }
        put(c + "-dasha-count", chart.dashas().size());
        Ashtakavarga av = chart.ashtakavarga().orElseThrow();
        put(c + "-ashtakavarga", av.shodhana().key() + " " + av.ekadhipatya().key());
        for (GrahaAshtakavarga g : av.grahas()) {
            String key = c + "-ashtakavarga-" + g.graha().fullKey();
            put(key, ints(",", g.bindus()));
            put(key + "-reduced", g.reduced() != null && !g.reduced().isEmpty() ? ints(",", g.reduced()) : null);
            put(key + "-pindas", g.rashiPinda() + "," + g.grahaPinda() + "," + g.yogaPinda());
        }
        put(c + "-sarvashtakavarga", ints(",", av.sarva()) + ";" + ints(",", av.trikona()) + ";" + ints(",", av.reduced()));
        for (GrahaShadbala strength : chart.shadbala().orElseThrow().grahas()) {
            String key = c + "-shadbala-" + strength.graha().fullKey();
            SthanaBala st = strength.sthana();
            KaalaBala ka = strength.kaala();
            List<Double> parts = List.of(st.uchcha(), st.saptavargaja(), st.ojayugma(), st.kendradi(), st.drekkana(),
                    strength.dig(), ka.nathonnatha(), ka.paksha(), ka.tribhaga(), ka.abda(), ka.masa(), ka.vara(),
                    ka.hora(), ka.ayana(), ka.yuddha(), strength.cheshta(), strength.naisargika(), strength.drik());
            put(key, numbers(",", parts));
            put(key + "-total", number(strength.virupas()) + "," + number(strength.rupas()) + ","
                    + number(strength.requiredRupas()) + "," + lower(strength.strong()) + "," + number(strength.ishta())
                    + "," + number(strength.kashta()) + "," + number(strength.subhaRashmi()) + ","
                    + number(strength.ashubhaRashmi()));
        }
        for (BhavaStrength house : chart.bhavaBala().orElseThrow().bhavas()) {
            List<Double> values = List.of(house.adhipati(), house.dig(), house.drishti(), house.special(), house.virupas());
            put(c + "-bhava-bala-" + house.bhava(), house.lord().fullKey() + " " + numbers(",", values));
        }
        for (GrahaVaiseshikamsa named : chart.vaiseshikamsa().orElseThrow().grahas()) {
            List<VaiseshikamsaStanding> standings = List.of(named.shadvarga(), named.saptavarga(), named.dashavarga(),
                    named.shodashavarga());
            String said = join(",", standings, st -> st.goodVargas() + ":" + (st.name() != null ? st.name().fullKey() : "null"));
            put(c + "-vaiseshikamsa-" + named.graha().fullKey(), said + " " + lower(named.impaired()));
        }
        for (GrahaDashaPhala phala : chart.dashaPhala().orElseThrow().grahas()) {
            put(c + "-dasha-phala-" + phala.graha().fullKey(), numbers(",", phala.subhankas()) + " "
                    + phala.nature().fullKey() + " " + phala.phase().key() + " " + lower(phala.favourable()) + " "
                    + lower(phala.unfavourable()));
        }
        JaiminiReading jr = chart.jaimini().orElseThrow();
        Karakamsha karakamsha = jr.karakamsha();
        Brahma brahma = jr.brahma();
        put(c + "-jaimini", karakamsha.atmakaraka().fullKey() + " " + karakamsha.sign().fullKey() + " "
                + ints(",", karakamsha.inRasi()) + " " + ints(",", karakamsha.inNavamsha()));
        put(c + "-graha-arudhas", join(",", jr.grahaArudhas(), sign -> sign != null ? sign.fullKey() : "-"));
        Avakahada birth = chart.avakahada().orElseThrow();
        put(c + "-avakahada", birth.nakshatra().fullKey() + " " + birth.pada() + " " + birth.rashi().fullKey() + " "
                + birth.nakshatraLord().fullKey() + " " + birth.rashiLord().fullKey() + " " + birth.varna().fullKey()
                + " " + birth.yoni().fullKey() + " " + birth.gana().fullKey() + " " + birth.nadi().fullKey());
        BirthSyllable syllable = birth.syllable();
        put(c + "-avakahada-syllable", syllable.cell() + " " + syllable.devanagari() + " " + syllable.iast() + " "
                + syllable.varga().key());
        put(c + "-brahma", brahma.rule().key() + " " + brahma.countedFrom().fullKey() + " "
                + grahaKeys(brahma.qualified())
                + " " + (brahma.graha() != null ? brahma.graha().fullKey() : "-")
                + " " + (brahma.passedFrom() != null ? brahma.passedFrom().fullKey() : "-")
                + " " + (brahma.none() != null ? brahma.none().key() : "-"));
        for (int slot = 0; slot < chart.gochar().size(); slot += 1) {
            GocharReading transit = chart.gochar().get(slot);
            GocharReference ref = transit.reference();
            GocharRules rules = transit.rules();
            String at = c + "-gochar-" + slot;
            put(at, number(transit.instant()) + " " + ref.from().key() + " " + ref.sign().fullKey() + " "
                    + rules.nodeVedha().key() + " " + rules.nodeObstruction().key() + " "
                    + rules.ashtakavargaGoodFrom().key());
            List<AshtakavargaTransit> judgedAll = transit.ashtakavarga() == null ? List.of() : transit.ashtakavarga();
            for (int k = 0; k < judgedAll.size(); k += 1) {
                AshtakavargaTransit judged = judgedAll.get(k);
                put(at + "-av-" + k, judged.graha().fullKey() + " " + judged.bindus() + " " + lower(judged.good()) + " "
                        + judged.kakshya().index() + " " + judged.kakshya().lord().key() + " "
                        + lower(judged.kakshyaBindu()) + " " + judged.sarva() + " " + judged.sarvaStanding().key());
            }
            for (int k = 0; k < transit.grahas().size(); k += 1) {
                GrahaGochar moving = transit.grahas().get(k);
                put(at + "-" + k, moving.graha().fullKey() + " " + moving.transit().sign().fullKey() + " "
                        + number(moving.transit().degrees()) + " " + moving.house() + " " + lower(moving.goodHouse())
                        + " " + (moving.vedhaHouse() != null ? String.valueOf(moving.vedhaHouse()) : "-")
                        + " " + grahaKeys(moving.obstructedBy())
                        + " " + moving.verdict().key() + " " + moving.fruition().key() + " "
                        + lower(moving.fruitfulNow()));
            }
        }
        for (int k = 0; k < chart.hits().size(); k += 1) {
            Hit hit = chart.hits().get(k);
            HitEvent e = hit.event();
            String into = "-";
            String motion;
            String to = "-";
            String angle = "-";
            String phase = "-";
            if (e instanceof SignIngress sign) {
                into = sign.into().fullKey();
                motion = sign.motion().key();
            } else if (e instanceof NakshatraIngress star) {
                into = star.into().fullKey();
                motion = star.motion().key();
            } else if (e instanceof Station station) {
                motion = station.turns().key();
            } else if (e instanceof AspectHit aspect) {
                motion = aspect.motion().key();
                to = natalKey(aspect.to());
                angle = String.valueOf(aspect.angle());
                phase = aspect.phase().key();
            } else {
                throw new IllegalStateException("a hit this runner does not know: " + e);
            }
            put(c + "-hit-" + k, number(hit.instant()) + " " + hit.graha().fullKey() + " " + e.kind().key() + " " + into
                    + " " + motion + " " + to + " " + angle + " " + phase);
        }
        SadeSatiReport ss = chart.sadeSati().orElseThrow();
        put(c + "-sade-sati", ss.reference().from().key() + " " + ss.reference().sign().fullKey() + " "
                + ss.reckoning().key());
        List<List<SadeSatiSpell>> periods = new ArrayList<>();
        for (SadeSati one : ss.sadeSati()) {
            periods.add(one.phases());
        }
        for (SadeSatiSpell spell : ss.spells()) {
            periods.add(List.of(spell));
        }
        List<String> lines = new ArrayList<>();
        for (int period = 0; period < periods.size(); period += 1) {
            for (SadeSatiSpell spell : periods.get(period)) {
                for (SadeSatiVisit v : spell.visits()) {
                    lines.add(period + " " + spell.house() + " " + bound(v.from()) + " " + bound(v.to()));
                }
            }
        }
        for (int k = 0; k < lines.size(); k += 1) {
            put(c + "-sade-sati-" + k, lines.get(k));
        }
        kp(c, chart.kp().orElseThrow());
        western(c, chart);
        put(c + "-dasha-count", chart.dashas().size());
        dashas(c, chart);
        for (int v = 0; v < chart.vargas().size(); v += 1) {
            VargaChart varga = chart.vargas().get(v);
            String at = c + "-varga-" + v;
            put(at, varga.varga().fullKey());
            put(at + "-lagna-rashi", varga.lagna().rashi().fullKey());
            put(at + "-lagna-part", varga.lagna().part());
            put(at + "-lagna-sign", varga.lagna().sign().fullKey());
            for (int j = 0; j < varga.grahas().size(); j += 1) {
                PlacedInVarga inVarga = varga.grahas().get(j);
                put(at + "-graha-" + j, inVarga.graha().fullKey());
                put(at + "-graha-" + j + "-rashi", inVarga.at().rashi().fullKey());
                put(at + "-graha-" + j + "-part", inVarga.at().part());
                put(at + "-graha-" + j + "-sign", inVarga.at().sign().fullKey());
            }
        }
        for (int j = 0; j < chart.grahas().size(); j += 1) {
            PlacedGraha graha = chart.grahas().get(j);
            String at = c + "-graha-" + j;
            put(at, graha.graha().fullKey());
            put(at + "-lon", graha.longitudeDeg());
            put(at + "-lat", graha.latitudeDeg());
            put(at + "-speed", graha.speedDegPerDay());
            put(at + "-retro", graha.retrograde());
            put(at + "-house", graha.house().bhava());
            put(at + "-house-method", graha.house().method().fullKey());
            put(at + "-placement", graha.placement().bhava());
        }
        // Uranus, Neptune and Pluto, which the request asks beside the nine.
        for (int j = 0; j < chart.outer().size(); j += 1) {
            PlacedGraha outer = chart.outer().get(j);
            put(c + "-outer-" + j, outer.graha().fullKey() + " " + number(outer.longitudeDeg()) + " "
                    + number(outer.latitudeDeg()) + " " + number(outer.speedDegPerDay()) + " " + outer.house().bhava()
                    + " " + outer.placement().bhava());
        }
        for (int k = 0; k < chart.houses().size(); k += 1) {
            put(c + "-house-" + k + "-madhya", chart.houses().get(k).madhyaDeg());
            put(c + "-house-" + k + "-sandhi", chart.houses().get(k).sandhiDeg());
        }
        for (int k = 0; k < chart.chalit().size(); k += 1) {
            put(c + "-chalit-" + k + "-madhya", chart.chalit().get(k).madhyaDeg());
        }
    }

    /** A chart's KP reading. */
    private static void kp(String c, KpReading kp) {
        KpRulingRules rules = kp.ruling().rules();
        put(c + "-kp", kp.chart().system().fullKey() + " " + rules.count() + " " + rules.nodeRulers() + " "
                + rules.retrogradeRejection());
        for (KpCusp cusp : kp.chart().cusps()) {
            put(c + "-kp-cusp-" + cusp.house(), cusp.longitude() + " " + lords(cusp.lords()));
        }
        for (KpPlanet p : kp.chart().planets()) {
            put(c + "-kp-planet-" + p.graha().fullKey(), p.longitude() + " " + lower(p.retrograde()) + " " + p.house()
                    + " " + lords(p.lords()));
        }
        for (KpHouseSignificators h : kp.significators().houses()) {
            put(c + "-kp-house-" + h.house(), grahaKeys(h.inOccupantsStars()) + " " + grahaKeys(h.occupants()) + " "
                    + grahaKeys(h.inLordsStar()) + " " + h.lord().fullKey() + " " + grahaKeys(h.conjoined()) + " "
                    + grahaKeys(h.aspected()) + " " + orDash(join(",", h.intercepted(), Rashi::fullKey)));
        }
        for (KpNodeAgency agency : kp.significators().nodes()) {
            put(c + "-kp-node-" + agency.node().fullKey(), grahaKeys(agency.conjoined()) + " "
                    + agency.starLord().fullKey() + " " + grahaKeys(agency.aspecting()) + " "
                    + agency.signLord().fullKey());
        }
        for (int k = 0; k < kp.ruling().rulers().size(); k += 1) {
            KpRuler r = kp.ruling().rulers().get(k);
            String reasons = join(",", r.reasons(),
                    why -> why.of() != null ? "AGENT:" + why.of().fullKey() + ":" + why.by() : why.kind());
            put(c + "-kp-ruler-" + k, r.graha().fullKey() + " " + reasons + " " + lower(r.retrograde()) + " "
                    + rejection(r.rejectedBy()) + " " + rejection(r.rejectedBySub()));
        }
    }

    /** A chart's Western and horary readings, matching and synastry. */
    private static void western(String c, Chart chart) {
        Dignities dg = chart.dignities().orElseThrow();
        DignityScores sc = dg.scores();
        List<Integer> worth = List.of(sc.house(), sc.exaltation(), sc.triplicity(), sc.term(), sc.face(),
                sc.detriment(), sc.fall(), sc.peregrine());
        put(c + "-dignities", dg.sect().key() + " " + dg.sectRule().key() + " " + dg.rules().terms().key() + " "
                + dg.rules().triplicities().key() + " " + ints(",", worth));
        for (PlanetDignity planet : dg.planets()) {
            List<String> held = dignityFlags(planet.dignity());
            if (planet.peregrine()) {
                held.add("peregrine");
            }
            put(c + "-dignity-" + planet.planet().fullKey(), number(planet.longitudeDeg()) + " "
                    + orDash(String.join(",", held)) + " " + planet.score() + " " + planet.reception());
        }
        for (int k = 0; k < dg.receptions().size(); k += 1) {
            Reception reception = dg.receptions().get(k);
            put(c + "-reception-" + k, reception.planets().get(0).fullKey() + " " + reception.planets().get(1).fullKey()
                    + " " + String.join(",", dignityFlags(reception.firstIn())) + " "
                    + String.join(",", dignityFlags(reception.secondIn())) + " "
                    + orDash(String.join(",", reception.mutual())));
        }
        Fortitudes ft = chart.fortitudes().orElseThrow();
        AccidentalSky fsky = ft.sky();
        put(c + "-fortitudes", fsky.houses().fullKey() + " " + numbers(" ", List.of(fsky.northNodeDeg(),
                fsky.regulusDeg(), fsky.spicaDeg(), fsky.algolDeg())));
        AccidentalRules fr = ft.rules();
        String partile = fr.partile().key().equals("WITHIN") ? "WITHIN:" + number(fr.partileOrbDeg()) : fr.partile().key();
        String siege = fr.siege().key().equals("WITHIN") ? "WITHIN:" + number(fr.siegeSpanDeg()) : fr.siege().key();
        put(c + "-fortitude-rules", number(fr.combustionDeg()) + " " + flag(fr.combustionInSign()) + " "
                + numbers(" ", List.of(fr.beamsDeg(), fr.cazimiDeg(), fr.cuspOrbDeg(), fr.starOrbDeg())) + " "
                + partile + " " + siege + " " + numbers(",", fr.meanMotionDeg()));
        AccidentalScores s = ft.scores();
        // Every field of the scores but `houses`, in the record's order, as
        // Python's `dataclass_fields` walks them.
        List<Integer> linesScored = List.of(s.direct(), s.retrograde(), s.swift(), s.slow(), s.superiorOriental(),
                s.superiorOccidental(), s.inferiorOriental(), s.inferiorOccidental(), s.increasing(), s.decreasing(),
                s.freeFromCombustion(), s.cazimi(), s.combust(), s.underBeams(), s.conjunctBenefic(),
                s.conjunctNorthNode(), s.trineBenefic(), s.sextileBenefic(), s.conjunctMalefic(),
                s.conjunctSouthNode(), s.opposedMalefic(), s.squareMalefic(), s.besieged(), s.regulus(), s.spica(),
                s.algol());
        put(c + "-fortitude-scores", ints(",", s.houses()) + " " + ints(",", linesScored));
        put(c + "-fortitude-houses", numbers(",", fsky.cuspsDeg()));
        for (int k = 0; k < ft.planets().size(); k += 1) {
            PlanetAccidents accidents = ft.planets().get(k);
            String met = join(",", accidents.accidents(), line -> line.accident().key() + ":" + line.points());
            put(c + "-fortitude-" + accidents.planet().fullKey(), number(fsky.speedsDegPerDay().get(k)) + " "
                    + accidents.house() + " " + orDash(met) + " " + accidents.fortitude() + " " + accidents.debility()
                    + " " + accidents.net());
        }
        Almutens al = ft.almutens();
        put(c + "-almuten-rules", al.rules().place().key() + " " + al.rules().fortune().key() + " "
                + numbers(" ", List.of(al.fortuneDeg(), fsky.ascendantDeg(), fsky.midheavenDeg())));
        put(c + "-almuten-figure", ranked(al.figure()));
        put(c + "-almuten-places", ranked(al.places()));
        for (int house = 1; house <= al.houses().size(); house += 1) {
            put(c + "-almuten-house-" + house, ranked(al.houses().get(house - 1)));
        }
        Lots lt = chart.lots().orElseThrow();
        put(c + "-lots", lt.sect().key() + " " + lt.request().sectRule().key() + " " + lt.request().fortune().key() + " "
                + flag(lt.fortuneReversed()));
        for (PlacedLot placedLot : lt.lots()) {
            LotPlace lotAt = placedLot.place();
            put(c + "-lot-" + placedLot.lot().key(), number(lotAt.longitudeDeg()) + " " + lotAt.sign().fullKey() + " "
                    + lotAt.lord().fullKey() + " " + lotAt.house());
        }
        Considerations cs = chart.considerations().orElseThrow();
        Radicality rd = cs.radicality();
        AscendantClause asc = cs.ascendant();
        MoonClause mn = cs.moon();
        SeventhClause sv = cs.seventh();
        put(c + "-considerations", rd.hourLord().fullKey() + " " + rd.ascendantLord().fullKey() + " "
                + orDash(join(",", rd.grounds(), RadicalGround::key)) + " " + asc.sign().fullKey() + " "
                + number(asc.degree()) + " " + flag(asc.early()) + " " + flag(asc.late()) + " "
                + flag(asc.shortAscension()));
        put(c + "-considerations-moon", mn.sign().fullKey() + " " + number(mn.degree()) + " " + flag(mn.late()) + " "
                + flag(mn.lateSign()) + " " + flag(mn.viaCombusta()) + " " + number(mn.course().daysInSign()) + " "
                + flag(mn.course().eased()));
        put(c + "-considerations-next", perfectionSaid(mn.course().next()));
        put(c + "-considerations-within", perfectionSaid(mn.course().withinOrb()));
        put(c + "-considerations-seventh", number(sv.cuspDeg()) + " " + sv.lord().fullKey() + " "
                + grahaKeys(sv.infortunesInHouse()) + " " + flag(sv.lordRetrograde()) + " " + flag(sv.lordCombust())
                + " " + flag(sv.lordInFall()) + " " + flag(sv.lordInInfortuneTerm()) + " " + sv.lordNet());
        put(c + "-considerations-saturn", cs.saturnHouse() + " " + flag(cs.saturnRetrograde()) + " "
                + flag(cs.ascendantLordCombust()));
        put(c + "-considerations-rules", number(cs.rules().moonLateFromDeg()) + " "
                + numbers(",", cs.rules().orbsDeg()));
        perfection(c, chart.perfection().orElseThrow());
        prashna(c, chart.prashna().orElseThrow());
        putRemedies(c + "-remedies", chart.remedies().orElseThrow());
        progressions(c, chart.progressions().orElseThrow());
        List<WesternAspectRow> westernAspects = chart.westernAspects().orElseThrow();
        put(c + "-western-aspect-count", String.valueOf(westernAspects.size()));
        for (int n = 0; n < westernAspects.size(); n += 1) {
            WesternAspectRow row = westernAspects.get(n);
            put(c + "-western-aspect-" + n, row.first().fullKey() + " " + row.second().fullKey() + " " + row.aspect().key()
                    + " " + number(row.apartDeg()) + " " + number(row.fromExactDeg()) + " " + number(row.orbDeg()) + " "
                    + flag(row.applying()));
        }
        Declinations declined = chart.declinations().orElseThrow();
        put(c + "-declinations", number(declined.obliquityDeg()) + " " + number(declined.lagnaDeg()) + " "
                + number(declined.midheavenDeg()));
        for (Declined at : declined.grahas()) {
            put(c + "-declination-" + at.graha().fullKey(), number(at.declinationDeg()));
        }
        List<ParallelRow> parallels = chart.parallels().orElseThrow();
        put(c + "-parallel-count", String.valueOf(parallels.size()));
        for (int n = 0; n < parallels.size(); n += 1) {
            ParallelRow pair = parallels.get(n);
            put(c + "-parallel-" + n, pair.first().fullKey() + " " + pair.second().fullKey() + " " + flag(pair.contrary())
                    + " " + number(pair.apartDeg()) + " " + number(pair.orbDeg()));
        }
        Antiscia reflected = chart.antiscia().orElseThrow();
        for (Antiscion reflection : reflected.points()) {
            put(c + "-antiscion-" + reflection.graha().fullKey(), number(reflection.antiscionDeg()) + " "
                    + number(reflection.contrantiscionDeg()));
        }
        put(c + "-antiscia-unpaired", grahaKeys(reflected.unpaired()));
        putAntiscionRows(c + "-antiscia", reflected.pairs());
        HouseSystem cuspSystem = Objects.requireNonNull(reflected.cuspSystem(), "the antiscia name no cusp system");
        put(c + "-antiscia-cusps", cuspSystem.fullKey() + " " + reflected.onCusps().size());
        for (int n = 0; n < reflected.onCusps().size(); n += 1) {
            CuspAntiscion upon = reflected.onCusps().get(n);
            put(c + "-antiscia-cusp-" + n, upon.graha().fullKey() + " " + upon.house() + " " + flag(upon.contrary()));
        }
        WesternHouses houses = chart.westernHouses().orElseThrow();
        put(c + "-western-houses", houses.system().fullKey() + " " + number(houses.ascendantDeg()) + " "
                + number(houses.reachDeg()) + " " + houses.planets().size());
        for (int n = 1; n <= houses.cuspsDeg().size(); n += 1) {
            put(c + "-western-cusp-" + n, number(houses.cuspsDeg().get(n - 1)));
        }
        for (WesternHousePlacement counted : houses.planets()) {
            put(c + "-western-house-" + counted.graha().fullKey(), counted.house() + " " + flag(counted.withAscendant()));
        }
        putAshta(c, chart.matching().orElseThrow());
        putPorutham(c, chart.porutham().orElseThrow());
        Kuja mars = chart.kuja().orElseThrow();
        Map<String, KujaSide> sides = new LinkedHashMap<>();
        sides.put("bride", mars.bride());
        sides.put("groom", mars.groom());
        for (Map.Entry<String, KujaSide> side : sides.entrySet()) {
            String readings = join(" ", side.getValue().readings(),
                    r -> r.reference() + " " + r.house() + " " + flag(r.inHouses()));
            put(c + "-kuja-" + side.getKey(), readings + " " + flag(side.getValue().dosha()));
        }
        put(c + "-kuja", flag(mars.both()));
        List<MarriageDosha> doshas = chart.marriageDoshas().orElseThrow();
        put(c + "-doshas", String.valueOf(doshas.size()));
        for (int n = 0; n < doshas.size(); n += 1) {
            MarriageDosha dosha = doshas.get(n);
            String named = dosha.koota() == null ? "NONE" : dosha.koota().fullKey();
            String carrier = dosha.side() == null ? "NONE" : dosha.side().key();
            put(c + "-dosha-" + n, dosha.system().key() + " " + named + " " + carrier + " " + flag(dosha.lifted()));
        }
        HarmonicChart fifth = chart.harmonic().orElseThrow();
        put(c + "-harmonic", fifth.harmonic() + " " + fifth.points().size() + " " + fifth.rows().size());
        for (HarmonicPlaced raised : fifth.points()) {
            put(c + "-harmonic-" + harmonicKey(raised.point()), number(raised.longitudeDeg()) + " " + raised.house());
        }
        for (int n = 0; n < fifth.rows().size(); n += 1) {
            HarmonicRow meeting = fifth.rows().get(n);
            put(c + "-harmonic-row-" + n, harmonicKey(meeting.first()) + " " + harmonicKey(meeting.second()) + " "
                    + number(meeting.apartDeg()) + " " + meeting.multiple() + " " + number(meeting.orbDeg()));
        }
        List<MidpointRow> between = chart.midpoints().orElseThrow();
        put(c + "-midpoint-count", String.valueOf(between.size()));
        for (int n = 0; n < between.size(); n += 1) {
            MidpointRow equal = between.get(n);
            put(c + "-midpoint-" + n, equal.first().fullKey() + " " + equal.second().fullKey() + " "
                    + equal.middle().fullKey() + " " + flag(equal.far()) + " " + number(equal.distanceDeg()) + " "
                    + number(equal.fromAxisDeg()) + " " + number(equal.orbDeg()));
        }
        List<SynastryRow> synastry = chart.synastry().orElseThrow();
        put(c + "-synastry-count", String.valueOf(synastry.size()));
        for (int n = 0; n < synastry.size(); n += 1) {
            SynastryRow across = synastry.get(n);
            put(c + "-synastry-" + n, natalKey(across.first()) + " " + natalKey(across.second()) + " "
                    + across.aspect().key() + " " + number(across.apartDeg()) + " " + number(across.fromExactDeg()) + " "
                    + number(across.orbDeg()));
        }
        List<SynastryParallelRow> levelled = chart.synastryParallels().orElseThrow();
        put(c + "-synastry-parallel-count", String.valueOf(levelled.size()));
        for (int n = 0; n < levelled.size(); n += 1) {
            SynastryParallelRow level = levelled.get(n);
            put(c + "-synastry-parallel-" + n, natalKey(level.first()) + " " + natalKey(level.second()) + " "
                    + flag(level.contrary()) + " " + number(level.apartDeg()) + " " + number(level.orbDeg()));
        }
        putAntiscionRows(c + "-synastry-antiscia", chart.synastryAntiscia().orElseThrow());
        List<SynastryMidpointRow> acrossEqual = chart.synastryMidpoints().orElseThrow();
        put(c + "-synastry-midpoint-count", String.valueOf(acrossEqual.size()));
        for (int n = 0; n < acrossEqual.size(); n += 1) {
            SynastryMidpointRow pair = acrossEqual.get(n);
            put(c + "-synastry-midpoint-" + n, pair.first().fullKey() + " " + pair.second().fullKey() + " "
                    + pair.middle().fullKey() + " " + flag(pair.partnersPair()) + " " + flag(pair.far()) + " "
                    + number(pair.distanceDeg()) + " " + number(pair.fromAxisDeg()) + " " + number(pair.orbDeg()));
        }
        Composite composite = chart.synastryComposite().orElseThrow();
        put(c + "-composite", number(composite.lagnaDeg()) + " " + number(composite.midheavenDeg()) + " "
                + flag(composite.lagnaTurned()) + " " + composite.planets().size());
        for (int n = 0; n < composite.planets().size(); n += 1) {
            CompositePlanet middle = composite.planets().get(n);
            put(c + "-composite-" + n, middle.graha().fullKey() + " " + number(middle.longitudeDeg()) + " "
                    + number(middle.speedDegPerDay()));
        }
        put(c + "-composite-cusps", composite.cuspsDeg() == null ? "-" : numbers(" ", composite.cuspsDeg()));
        DavisonBirth davison = chart.synastryDavison().orElseThrow();
        put(c + "-davison", number(davison.instant()) + " " + number(davison.place().latitudeDeg().value()) + " "
                + number(davison.place().longitudeDeg().value()) + " " + number(davison.place().altitudeM().value())
                + " " + davison.utcOffsetSeconds());
        Vimshopaka vs = chart.vimshopaka().orElseThrow();
        put(c + "-vimshopaka", vs.scoring().key());
        for (GrahaVimshopaka scored : vs.grahas()) {
            put(c + "-vimshopaka-" + scored.graha().fullKey(), numbers(",", List.of(scored.shadvarga(),
                    scored.saptavarga(), scored.dashavarga(), scored.shodashavarga())));
        }
    }

    /** A chart's horary perfection. */
    private static void perfection(String c, Matter pf) {
        put(c + "-perfection", pf.querent().fullKey() + " " + pf.quesited().fullKey() + " " + number(pf.horizonDays())
                + " " + pf.impediments().size() + " " + pf.translations().size() + " " + pf.collections().size());
        Application ap = pf.application();
        put(c + "-perfection-application", ap == null
                ? "-"
                : ap.aspect().key() + " " + number(ap.days()) + " " + ap.applying().fullKey() + " " + ap.kind().key()
                        + " " + number(ap.gapDeg()) + " " + flag(ap.withinMoieties()));
        Separation sp = pf.separation();
        put(c + "-perfection-separation", sp == null ? "-" : sp.aspect().key() + " " + number(sp.pastDeg()));
        Ways wy = pf.ways();
        put(c + "-perfection-ways", wy.querent().house() + " " + dignitiesHeld(wy.querent().dignity()) + " "
                + wy.quesited().house() + " " + dignitiesHeld(wy.quesited().dignity()) + " " + flag(wy.mutualByHouse())
                + " " + grahaKeys(wy.infortunesBetween()) + " " + flag(wy.moonRelays()) + " "
                + flag(wy.quesitedInAscendant()) + " " + orDash(join(",", wy.held(), Way::key)));
        for (int n = 0; n < pf.impediments().size(); n += 1) {
            Impediment im = pf.impediments().get(n);
            put(c + "-perfection-impediment-" + n, im.kind().key() + " " + im.significator().fullKey() + " "
                    + (im.third() != null ? im.third().fullKey() : "-") + " " + im.aspect().key() + " "
                    + number(im.days()));
        }
        for (int n = 0; n < pf.translations().size(); n += 1) {
            Translation tr = pf.translations().get(n);
            put(c + "-perfection-translation-" + n, tr.translator().fullKey() + " " + tr.from().fullKey() + " "
                    + tr.to().fullKey() + " " + tr.separating().aspect().key() + " " + number(tr.separating().pastDeg())
                    + " " + tr.aspect().key() + " " + number(tr.days()) + " " + dignitiesHeld(tr.received()));
        }
        for (int n = 0; n < pf.collections().size(); n += 1) {
            Collection co = pf.collections().get(n);
            put(c + "-perfection-collection-" + n, co.collector().fullKey() + " " + co.fromQuerent().aspect().key() + " "
                    + number(co.fromQuerent().days()) + " " + co.fromQuesited().aspect().key() + " "
                    + number(co.fromQuesited().days()) + " " + dignitiesHeld(co.collectorInQuerent()) + " "
                    + dignitiesHeld(co.collectorInQuesited()) + " " + dignitiesHeld(co.querentInCollector()) + " "
                    + dignitiesHeld(co.quesitedInCollector()));
        }
        put(c + "-perfection-rules", numbers(",", pf.rules().orbsDeg()) + " " + flag(pf.rules().withinSign()));
    }

    /** A chart's prashna. */
    private static void prashna(String c, Prashna pq) {
        PrashnaLinks links = Objects.requireNonNull(pq.links(), "the prashna has no links");
        PrashnaScore ps = Objects.requireNonNull(pq.score(), "the prashna has no score");
        Map<String, Object> rq = pq.rules();
        put(c + "-prashna", py(dig(rq, "pisces")) + " " + py(dig(rq, "timing")) + " " + py(dig(rq, "mook")) + " "
                + py(dig(rq, "moon", "kshina")) + " " + py(dig(rq, "score")) + " " + pq.verdict().outcome() + " "
                + pq.change() + " " + (pq.numberSign() != null ? pq.numberSign().fullKey() : "-"));
        for (int n = 0; n < pq.verdict().clauses().size(); n += 1) {
            PrashnaClause clause = pq.verdict().clauses().get(n);
            String about = clause.graha() != null ? clause.graha().fullKey() : "-";
            put(c + "-prashna-clause-" + n, clause.kind() + " " + about + " " + clause.favour());
        }
        PrashnaTiming tm = pq.timing();
        put(c + "-prashna-timing", tm.rule() + " " + tm.graha().fullKey() + " " + lower(tm.tie()) + " " + tm.count() + " "
                + tm.multiplier() + " " + (tm.amount() == null ? "-" : String.valueOf(tm.amount())) + " " + tm.unit()
                + " " + grahaKeys(tm.between()));
        PrashnaMook mk = pq.mook();
        put(c + "-prashna-mook", mk.rule() + " " + mk.graha().fullKey() + " " + lower(mk.tie()) + " " + mk.house() + " "
                + (mk.person() == null || mk.person().isEmpty() ? "-" : mk.person()) + " " + mk.thought());
        put(c + "-prashna-moon", orDash(String.join(",", pq.moon().clauses())));
        put(c + "-prashna-score", ps.points() + " " + ps.answer() + " " + lower(ps.isVoid()) + " "
                + (ps.applyingTo() != null ? ps.applyingTo().fullKey() : "-") + " "
                + orDash(join(",", ps.factors(), f -> f.kind() + ":" + f.points())));
        putMatter(c + "-prashna-links", links.matter());
        AnnualStatesRead states = Objects.requireNonNull(links.states(), "the prashna's links have no states");
        put(c + "-prashna-links-states", "R:" + grahaKeys(states.retrograde()) + " C:" + grahaKeys(states.combust()));
    }

    /** A chart's progressions and directions. */
    private static void progressions(String c, Progressions pr) {
        Progressed pg = Objects.requireNonNull(pr.progressed(), "no progressed chart");
        Directed dr = Objects.requireNonNull(pr.directed(), "no directed chart");
        List<ProgressedContact> contacts = Objects.requireNonNull(pr.contacts(), "no progressed contacts");
        put(c + "-progressed", number(pg.life()) + " " + number(pg.sky()) + " " + number(pg.armcDeg()) + " "
                + number(pg.angles().ascendantDeg()) + " " + number(pg.angles().midheavenDeg()));
        for (int n = 0; n < pg.grahas().size(); n += 1) {
            ProgressedPlanet gr = pg.grahas().get(n);
            put(c + "-progressed-graha-" + n, gr.graha().fullKey() + " " + number(gr.longitudeDeg()) + " "
                    + number(gr.tropicalDeg()) + " " + number(gr.speedDegPerDay()));
        }
        put(c + "-directed", number(dr.arcDeg()) + " " + number(dr.ascendantDeg()) + " " + number(dr.midheavenDeg()));
        for (int n = 0; n < dr.planets().size(); n += 1) {
            DirectedPlanet directed = dr.planets().get(n);
            put(c + "-directed-graha-" + n, directed.graha().fullKey() + " " + number(directed.longitudeDeg()));
        }
        put(c + "-progressed-contact-count", String.valueOf(contacts.size()));
        for (int n = 0; n < contacts.size(); n += 1) {
            ProgressedContact ct = contacts.get(n);
            put(c + "-progressed-contact-" + n, number(ct.life()) + " " + number(ct.sky()) + " " + ct.graha().fullKey()
                    + " " + natalKey(ct.to()) + " " + ct.angle() + " " + ct.motion().key());
        }
    }

    /** A chart's dashas and their periods to the second level. */
    private static void dashas(String c, Chart chart) {
        for (int j = 0; j < chart.dashas().size(); j += 1) {
            Dasha dasha = chart.dashas().get(j);
            String key = c + "-dasha-" + j;
            DashaBalance balance = dasha.balance();
            if (dasha.system() instanceof String registered) {
                put(key, registered);
            } else if (dasha.system() instanceof DashaSystem shipped) {
                put(key, shipped.fullKey());
            }
            put(key + "-seed", dasha.seed() != null ? dasha.seed().fullKey() : null);
            put(key + "-first-lord", dasha.firstLord().fullKey());
            put(key + "-overflow", dasha.overflow());
            put(key + "-balance", balance != null ? balance.method().key() : null);
            put(key + "-remaining", balance != null ? (Object) balance.remaining() : null);
            put(key + "-balance-days", balance != null ? (Object) balance.days() : null);
            WrittenBalance written = balance != null ? balance.written() : null;
            put(key + "-balance-written", written != null
                    ? written.years() + "," + written.months() + "," + written.days() + "," + written.hours() + ","
                            + written.minutes()
                    : null);
            put(key + "-moon-span-from", dasha.moonSpan() != null ? (Object) dasha.moonSpan().fromJd() : null);
            put(key + "-moon-span-to", dasha.moonSpan() != null ? (Object) dasha.moonSpan().toJd() : null);
            put(key + "-depth", dasha.depth());
            put(key + "-periods", dasha.periods().size());
            for (int k = 0; k < dasha.periods().size(); k += 1) {
                DashaPeriod period = dasha.periods().get(k);
                if (period.level() > 2) {
                    continue;
                }
                String sign = period.sign() != null ? " " + period.sign().fullKey() : "";
                put(key + "-period-" + k, period.path() + sign + " " + period.lord().fullKey());
                put(key + "-period-" + k + "-from", period.span().fromJd());
                put(key + "-period-" + k + "-to", period.span().toJd());
            }
            put(key + "-at", join(",", dasha.at(chart.instant() + 5000), DashaPeriod::path));
        }
    }

    /** One chart's annual charts under one reading. */
    private static void varsha(Chart chart, int i, String reading) {
        List<Pravesha> returns = chart.praveshas();
        String c = "chart-" + i + "-varsha-" + reading;
        put(c + "-count", returns.size());
        for (TajikaSaham point : chart.sahams()) {
            put(c + "-natal-saham-" + point.saham().key(), sahamSaid(point));
        }
        for (Pravesha pravesha : returns) {
            String stem = c + "-" + pravesha.year();
            put(stem, pravesha.instant());
            put(stem + "-muntha", pravesha.muntha().sign().fullKey());
            put(stem + "-muntha-lord", pravesha.muntha().lord().fullKey());
            put(stem + "-muntha-deg", pravesha.muntha().longitudeDeg());
            AnnualChart annual = Objects.requireNonNull(pravesha.annual(), "a year with no annual chart");
            put(stem + "-annual-lagna", annual.lagnaDeg());
            put(stem + "-annual-by-day", annual.byDay());
            OfficeBearers b = annual.officeBearers();
            put(stem + "-annual-bearers", join(" ", List.of(b.muntha(), b.janmaLagna(), b.varshaLagna(), b.triRashi(),
                    b.dinaRatri()), Graha::fullKey));
            YearLord yearLord = annual.yearLord();
            put(stem + "-year-lord", yearLord.graha().fullKey());
            put(stem + "-year-lord-chosen", yearLord.chosen().key());
            put(stem + "-year-lord-bala", yearLord.vishwa().toString());
            put(stem + "-yogas", join(" ", annual.yogas(), p -> p.faster().fullKey() + ">" + p.slower().fullKey() + ":"
                    + p.drishti().key() + ":" + p.yoga().key() + ":" + fixed(p.apartDeg(), 6)));
            put(stem + "-states", "R:" + join(",", annual.retrograde(), Graha::fullKey) + " C:"
                    + join(",", annual.combust(), Graha::fullKey));
            for (TajikaMatter matter : annual.matters()) {
                putMatter(stem + "-matter-" + matter.house(), matter);
            }
            for (TajikaSaham point : annual.sahams()) {
                put(stem + "-saham-" + point.saham().key(), sahamSaid(point));
            }
            for (AnnualDasha yearDasha : annual.dashas()) {
                String said = stem + "-dasha-" + yearDasha.system().fullKey();
                DashaRing ring = yearDasha.ring();
                String left = ring.remaining() == null ? "null" : fixed(ring.remaining(), 9);
                put(said, (yearDasha.seed() != null ? yearDasha.seed().fullKey() : "-") + " " + ring.first() + " " + left
                        + " " + fixed(yearDasha.year().fromJd(), 9) + " " + fixed(yearDasha.year().toJd(), 9) + " | "
                        + join(" ", ring.shares(), s -> s.lord().fullKey() + "/"
                                + (s.sign() != null ? s.sign().fullKey() : "-") + "/" + fixed(s.weight(), 3)));
                put(said + "-periods", join(" ", yearDasha.periods(), p -> p.path() + ":" + p.lord().fullKey() + ":"
                        + (p.sign() != null ? p.sign().fullKey() : "-") + ":" + fixed(p.span().fromJd(), 9) + ":"
                        + fixed(p.span().toJd(), 9)));
            }
            put(stem + "-harsha", join(" ", annual.harsha(), h -> h.graha().fullKey() + ":" + h.house() + ":"
                    + flag(h.sthana()) + flag(h.uchchaSwakshetra()) + flag(h.striPurusha()) + flag(h.dinaRatri()) + ":"
                    + h.total() + ":" + h.grade().key()));
            put(stem + "-year-claims", join(" ", yearLord.claims(), cl -> cl.graha().fullKey() + ":" + cl.vishwa() + ":"
                    + cl.portfolios() + ":" + lower(cl.aspectsLagna())));
        }
    }

    /** The almanac, the muhurta searches, the festivals, the lunar years and the Nepal Sambat dates. */
    private static void almanac(Context geo, Observer place) {
        // Three days, because a day's lists are ragged and two consecutive
        // days with the same counts would not exercise the offsets.
        Almanac week = geo.almanac().of(date(2024, 6, 17), date(2024, 6, 19), place, 20700);
        put("almanac-days", week.size());
        put("almanac-calendar", week.calendar().fullKey());
        put("almanac-place-lat", week.decoded().latitudeDeg());
        put("almanac-model-fnv", fnv(week.model()));
        put("almanac-provenance-fnv", fnv(week.provenanceJson()));
        for (AlmanacDay almanacDay : week) {
            int i = almanacDay.index();
            String d = "day-" + i;
            put(d + "-content-hash", almanacDay.provenance().contentHash());
            putDay(d, almanacDay.day());
            put(d + "-window-from", almanacDay.window().fromJd());
            put(d + "-window-to", almanacDay.window().toJd());
            put(d + "-month", almanacDay.month().month().fullKey());
            put(d + "-amanta", almanacDay.month().amanta().fullKey());
            put(d + "-purnimanta", almanacDay.month().purnimanta().fullKey());
            put(d + "-paksha", almanacDay.month().paksha().fullKey());
            put(d + "-convention", almanacDay.month().convention().key());
            put(d + "-month-kind", almanacDay.month().kind().key());
            put(d + "-ayana", almanacDay.ayana().fullKey());
            put(d + "-ritu", almanacDay.ritu().fullKey());
            put(d + "-disha-shool", almanacDay.dishaShool().fullKey());
            // An absent value must be absent in all, not nought in one.
            put(d + "-sankranti", almanacDay.sankranti().map(ParityRunner::number).orElse("none"));
            put(d + "-abhijit", almanacDay.abhijit().map(a -> number(a.at().fromJd())).orElse("none"));
            put(d + "-abhijit-effective", almanacDay.abhijit().map(a -> lower(a.effective())).orElse("none"));
            put(d + "-brahma", almanacDay.brahma().map(b -> number(b.fromJd())).orElse("none"));
            put(d + "-tithi-count", almanacDay.tithi().size());
            put(d + "-nakshatra-count", almanacDay.nakshatra().size());
            put(d + "-yoga-count", almanacDay.yoga().size());
            put(d + "-karana-count", almanacDay.karana().size());
            put(d + "-kaala-count", almanacDay.kaalas().size());
            put(d + "-choghadiya-count", almanacDay.choghadiya().size());
            put(d + "-hora-count", almanacDay.horas().size());
            put(d + "-muhurta-count", almanacDay.muhurtas().size());
            put(d + "-moon-event-count", almanacDay.moonEvents().size());
            put(d + "-panchaka-count", almanacDay.panchaka().size());
            put(d + "-moon-sign-count", almanacDay.moonSigns().size());
            put(d + "-sun-sign-count", almanacDay.sunSigns().size());
            put(d + "-muhurta-yoga-count", almanacDay.muhurtaYogas().size());
            putSpans(d + "-tithi", almanacDay.tithi(), Tithi::fullKey);
            putSpans(d + "-nakshatra", almanacDay.nakshatra(), Nakshatra::fullKey);
            putSpans(d + "-yoga", almanacDay.yoga(), Yoga::fullKey);
            putSpans(d + "-karana", almanacDay.karana(), Karana::fullKey);
            for (int j = 0; j < almanacDay.kaalas().size(); j += 1) {
                KaalaPeriod kaala = almanacDay.kaalas().get(j);
                put(d + "-kaala-" + j, kaala.kaala().fullKey());
                put(d + "-kaala-" + j + "-from", kaala.at().fromJd());
            }
            List<Hora> horas = almanacDay.horas();
            put(d + "-hora-0-lord", horas.get(0).lord().fullKey());
            put(d + "-hora-0-start", horas.get(0).start());
            put(d + "-hora-23-lord", horas.get(horas.size() - 1).lord().fullKey());
            List<ChoghadiyaPeriod> choghadiya = almanacDay.choghadiya();
            put(d + "-choghadiya-0", choghadiya.get(0).choghadiya().fullKey());
            put(d + "-choghadiya-0-daytime", choghadiya.get(0).daytime());
            List<Muhurta> muhurtas = almanacDay.muhurtas();
            put(d + "-muhurta-0-from", muhurtas.get(0).at().fromJd());
            put(d + "-muhurta-last-daylight", muhurtas.get(muhurtas.size() - 1).daylight());
            for (int j = 0; j < almanacDay.moonEvents().size(); j += 1) {
                MoonEventMoment event = almanacDay.moonEvents().get(j);
                put(d + "-moon-" + j + "-kind", event.rise() ? "RISE" : "SET");
                put(d + "-moon-" + j + "-instant", event.instant());
            }
            for (int j = 0; j < almanacDay.muhurtaYogas().size(); j += 1) {
                HeldYoga held = almanacDay.muhurtaYogas().get(j);
                put(d + "-yoga-held-" + j, held.yoga().fullKey());
                put(d + "-yoga-held-" + j + "-cause", held.tithi() == null ? "VARA_NAKSHATRA" : "VARA_TITHI_NAKSHATRA");
                put(d + "-yoga-held-" + j + "-tithi", held.tithi() == null ? "none" : held.tithi().fullKey());
            }
        }

        // `day` is the range of one unwrapped, and must agree.
        AlmanacDay oneDay = geo.almanac().day(date(2024, 6, 17), place, 20700);
        put("almanac-single-agrees", oneDay.day().sunrise() == week.at(0).day().sunrise());

        // ── A muhurta search ──────────────────────────────────────────────
        // Both rankings over 2024-11-25..27, and a thread ceremony, whose
        // rules want grahas out of houses.
        Map<String, Object> nativeBirth = map("star", "ROHINI", "moonSign", "TAURUS", "lagna", "LEO");
        Map<String, Map<String, Object>> searches = new LinkedHashMap<>();
        searches.put("raman", map("rules", "RAMAN_MARRIAGE", "ranking", "TEXTS"));
        searches.put("baseline", map("rules", "BASELINE_MARRIAGE", "ranking", "BASELINE"));
        searches.put("upanayana", map("rules", "RAMAN_UPANAYANA", "ranking", "TEXTS"));
        for (Map.Entry<String, Map<String, Object>> search : searches.entrySet()) {
            Map<String, Object> asked = new LinkedHashMap<>(search.getValue());
            asked.put("native", nativeBirth);
            asked.put("daysWithWindows", 3);
            asked.put("most", 12);
            MuhurtaAnswer muhurta = geo.almanac().of(date(2024, 11, 25), date(2024, 11, 27), place, 20700,
                    asked, null, false, false, false).muhurta().orElseThrow();
            putMuhurta("muhurta-" + search.getKey(), muhurta);
        }

        // ── Festivals ─────────────────────────────────────────────────────
        // The shipped pack, the pack amended by a rule of the consumer's own,
        // and the Nepal pack with a following rule of the consumer's own,
        // over 2024-10-10..11-03.
        Map<String, Object> ownRule = map(
                "key", "LAKSHMI_PUJA",
                "source", "the tithi at sunrise",
                "month", "masa.ASHWINA",
                "tithi", "tithi.AMAVASYA",
                "at", map("window", "SUNRISE"),
                "decide", List.of(),
                "otherwise", "LATER");
        Map<String, Object> ownFollowing = map(
                "key", "TWO_AFTER",
                "source", "two days after Lakshmi puja",
                "after", "LAKSHMI_PUJA",
                "days", 2);
        Map<String, Map<String, Object>> packs = new LinkedHashMap<>();
        packs.put("shipped", map("rules", "DHARMASINDHU"));
        packs.put("amended", map("rules", List.<Object>of("DHARMASINDHU", ownRule)));
        packs.put("nepal", map("rules", List.<Object>of("NEPAL", ownFollowing)));
        for (Map.Entry<String, Map<String, Object>> pack : packs.entrySet()) {
            FestivalAnswer festivals = geo.almanac().of(date(2024, 10, 10), date(2024, 11, 3), place, 20700,
                    null, pack.getValue(), false, false, false).festivals().orElseThrow();
            putFestivals("festivals-" + pack.getKey(), festivals);
        }

        // The lunar years over 2024-03-20..04-20, which holds a Chaitra Shukla
        // Pratipada: two years, their bounds and their Jovian years.
        LunarYears lunar = geo.almanac().of(date(2024, 3, 20), date(2024, 4, 20), place, 20700,
                null, null, true, false, false).years().orElseThrow();
        put("years-count", lunar.value().size());
        put("years-hash", lunar.provenance().contentHash());
        for (int k = 0; k < lunar.value().size(); k += 1) {
            LunarYear year = lunar.value().get(k);
            put("years-" + k, String.join(" ",
                    year.samvatsara().fullKey(),
                    year.count(),
                    String.valueOf(year.vikrama()),
                    String.valueOf(year.shaka()),
                    number(year.opened()),
                    number(year.began()),
                    number(year.ended()),
                    year.lupta() == null ? "-" : year.lupta().fullKey()));
            put("years-" + k + "-jovian",
                    orNone(join(" ", year.jovian(), j -> j.member().fullKey() + ":" + j.count() + ":" + number(j.from()))));
        }

        // The Nepal Sambat dates over 2024-10-30..11-03, which holds
        // Kartika's new moon, where the year turns.
        NepalSambatDates nepalSambat = geo.almanac().of(date(2024, 10, 30), date(2024, 11, 3), place, 20700,
                null, null, false, false, true).nepalSambat().orElseThrow();
        put("nepal-sambat-hash", nepalSambat.provenance().contentHash());
        put("nepal-sambat", orNone(join(" ", nepalSambat.value(),
                one -> one.year() + ":" + one.month() + ":" + one.kind().key() + ":" + one.paksha().fullKey())));
    }

    /** A day's spans of one limb, each under {@code <name>-<j>}. */
    private static <T> void putSpans(String name, List<Span<T>> spans, Function<? super T, String> key) {
        for (int j = 0; j < spans.size(); j += 1) {
            Span<T> span = spans.get(j);
            put(name + "-" + j, key.apply(span.member()));
            put(name + "-" + j + "-whole-from", span.whole().fromJd());
            put(name + "-" + j + "-inside-to", span.inside().toJd());
            put(name + "-" + j + "-sunrises", span.sunrises().key());
            put(name + "-" + j + "-ends", span.ends().ghati() + "-" + span.ends().pala() + "-" + span.ends().vipala());
        }
    }
}
