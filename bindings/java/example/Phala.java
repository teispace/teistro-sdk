import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.stream.Stream;

import com.teispace.teistro.Altitude;
import com.teispace.teistro.Chart;
import com.teispace.teistro.ChartOptions;
import com.teispace.teistro.Context;
import com.teispace.teistro.ContextOptions;
import com.teispace.teistro.Ephemeris;
import com.teispace.teistro.IntlArea;
import com.teispace.teistro.IntlLoaded;
import com.teispace.teistro.Json;
import com.teispace.teistro.Latitude;
import com.teispace.teistro.Longitude;
import com.teispace.teistro.Observer;
import com.teispace.teistro.SadeSatiReport;
import com.teispace.teistro.Teistro;
import com.teispace.teistro.blob.IntlRender;

/**
 * What the chart <i>is</i>, read aloud: the state readings, loaded beside
 * the rule readings.
 *
 * <p>{@code Readings.java} loaded a pack of readings for the yogas and
 * doshas a chart <b>holds</b>. This one loads the other half of that corpus:
 * a reading for what the chart <i>is</i> without holding anything — Jupiter
 * in the first house, the lagna's sign, the tithi, the nakshatra the Moon
 * stands in ({@code docs/03-design/state-readings.md}).
 *
 * <p>What it teaches:
 *
 * <ol>
 *   <li><b>Two packs, one engine.</b> Each root builds one pack a locale,
 *       and a consumer loads the ones it wants. They are loaded in either
 *       order.
 *   <li><b>A record gains forms; it does not lose them.</b> Both corpora
 *       describe some of the same subjects, and so may yours: a pack
 *       carrying one form adds that form and leaves the rest of the record
 *       standing. {@code loaded.merged()} counts the records that kept
 *       something.
 *   <li><b>A composer says nothing it has no words for.</b> {@code phala}
 *       asks the base locale for each subject and is silent where the
 *       answer is no, so a chart composes exactly as it did before until a
 *       pack is loaded.
 *   <li><b>A search is said from what it found.</b> {@code sadeSati} says
 *       the report a Sade Sati window beside it finds, in the same call:
 *       Saturn is scanned once, the report stays on the chart, and the plan
 *       says each house it holds once, in the corpus's words.
 * </ol>
 *
 * <p>The packs are the files {@code teistro-intl build} writes, one a
 * locale, which {@code cargo xtask check-parity} builds before it runs any
 * example:
 *
 * <pre>
 * cargo run -p teistro-intl -- --root packs/readings build --out target/packs/readings
 * cargo run -p teistro-intl -- --root packs/states build --out target/packs/states
 * </pre>
 */
public final class Phala {
    private Phala() {}

    /**
     * The two corpora, each built from its source under {@code packs/} into
     * packs of its own ({@code packs/README.md}).
     */
    private static final List<String> CORPORA = List.of("readings", "states");

    /**
     * Where the built packs are: {@code TEISTRO_PACKS}, or the repository's
     * {@code target/packs}, from {@code bindings/java} where the example runs.
     */
    private static final Path PACKS = Path.of(
            System.getenv("TEISTRO_PACKS") != null ? System.getenv("TEISTRO_PACKS") : "../../target/packs");

    /** Kathmandu, where the chart is founded. */
    private static final Observer KATHMANDU =
            new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));

    /**
     * Runs the example.
     *
     * @param args unused
     * @throws IOException when a pack cannot be read
     */
    public static void main(String[] args) throws IOException {
        Teistro teistro = Teistro.open();
        ContextOptions options = ContextOptions.builder()
                .profile("nepali-default")
                .ephemeris(Ephemeris.BUILTIN)
                .build();
        try (Context ctx = teistro.context(options)) {
            // ── The packs ────────────────────────────────────────────────
            for (String corpus : CORPORA) {
                for (Path file : packsOf(corpus)) {
                    byte[] data = Files.readAllBytes(file);
                    IntlLoaded loaded = ctx.intl().loadPack(data);
                    System.out.printf(Locale.ROOT, "%-15s %-12s %5d records, %4d merged, %7d bytes%n",
                            "packs/" + corpus, loaded.locale(), loaded.entries(), loaded.merged(), data.length);
                }
            }

            // ── The plan, said twice ─────────────────────────────────────
            // A composer is a member of `interpret` — `Map.of("phala", true)`
            // — off unless asked for, so a chart says nothing new until a
            // consumer asks for it. The chart is founded with what the
            // composer reads, its states and the panchanga's limbs among
            // them, in the same call.
            Chart chart = ctx.chart().found(2447995.4895833335, KATHMANDU, 20700, ChartOptions.builder()
                    .interpret(Map.of("phala", true))
                    .build());
            List<?> plan = plan(chart, "phala");
            System.out.println("\n" + plan.size() + " items");
            for (String locale : List.of("en-Latn", "ne-Deva-NP")) {
                ctx.intl().setLocale(locale);
                System.out.println("\n" + locale);
                for (Object item : plan.subList(0, Math.min(4, plan.size()))) {
                    IntlRender said = say(ctx.intl(), item);
                    String fallback = said.isFallback() != 0 ? "  (fallback)" : "";
                    System.out.println("  " + shortened(said.text()) + fallback);
                }
            }

            // ── Saturn's periods, said ───────────────────────────────────
            // `sadeSati` says the report a Sade Sati window found, which the
            // same call searches — once, for every chart asked — and leaves
            // on the chart beside the plan. Thirty years from 2000 hold a
            // whole Sade Sati and both smaller spells, and each house is
            // said once, in the order Saturn first reaches it.
            chart = ctx.chart().found(2447995.4895833335, KATHMANDU, 20700, ChartOptions.builder()
                    .sadeSati(Map.of("from", 2451545.0, "to", 2462502.5))
                    .interpret(Map.of("sadeSati", true))
                    .build());
            SadeSatiReport report = chart.sadeSati().orElseThrow();
            int periods = report.sadeSati().size() + report.spells().size();
            plan = plan(chart, "sadeSati");
            ctx.intl().setLocale("en-Latn");
            System.out.println("\nSade Sati: " + periods + " periods, " + plan.size() + " items");
            for (Object item : plan) {
                IntlRender said = say(ctx.intl(), item);
                Object house = ((Map<?, ?>) ((Map<?, ?>) item).get("params")).get("house");
                System.out.printf(Locale.ROOT, "  %2s  %s%n", house, shortened(said.text()));
            }

            // ── The record that two corpora describe ─────────────────────
            // `nakshatra-phala` says what the nakshatra portends and
            // `namakarana-nakshatra` what to name a child born under it.
            // Both are forms on the record the SDK already names, beside its
            // own `name` and `iast` — which is what the merge on load is
            // for. A pack's forms are read through `forms()`, since a
            // record's forms are an open set.
            ctx.intl().setLocale("en-Latn");
            Map<String, String> ashwini = ctx.intl().entity("nakshatra.ASHWINI").forms();
            System.out.println("\nnakshatra.ASHWINI");
            for (String form : List.of("name", "iast", "phala", "namakarana")) {
                if (ashwini.containsKey(form)) {
                    System.out.printf(Locale.ROOT, "  %-12s %s%n", form, shortened(ashwini.get(form)));
                }
            }

            // ── A reading no composer says ───────────────────────────────
            // Half the corpus is glossary rather than narrative: what it
            // means for a graha to be exalted is true of every exalted
            // graha, so no composer says it per chart. It is a record like
            // any other, and any catalogue key you can name you can ask for
            // — which is how a consumer builds a legend beside the plan.
            System.out.println("\ndignity.EXALTED");
            Map<String, String> exalted = ctx.intl().entity("dignity.EXALTED").forms();
            for (String form : List.of("name", "phala")) {
                if (exalted.containsKey(form)) {
                    System.out.printf(Locale.ROOT, "  %-12s %s%n", form, shortened(exalted.get(form)));
                }
            }
        }
    }

    /** Every pack a corpus built, one a locale, in name order. */
    private static List<Path> packsOf(String corpus) throws IOException {
        try (Stream<Path> files = Files.list(PACKS.resolve(corpus))) {
            return files.filter(file -> file.getFileName().toString().endsWith(".tpack")).sorted().toList();
        }
    }

    /** A plan the chart composed: its items, each an object of {@code key} and {@code params}. */
    private static List<?> plan(Chart chart, String name) {
        if (!(chart.plans().orElseThrow().get(name) instanceof List<?> items)) {
            throw new IllegalStateException("no `" + name + "` plan");
        }
        return items;
    }

    /** An item said in the context's locale: its params are what the renderer takes. */
    private static IntlRender say(IntlArea intl, Object item) {
        Map<?, ?> fields = (Map<?, ?>) item;
        return intl.renderJson(String.valueOf(fields.get("key")), Json.write(fields.get("params")));
    }

    /**
     * Enough of a passage to show it is there, without printing an essay; in
     * characters, not UTF-16 units, so a Devanagari passage is cut where
     * every binding cuts it.
     */
    private static String shortened(String text) {
        return text.codePointCount(0, text.length()) > 88
                ? text.substring(0, text.offsetByCodePoints(0, 88)) + "…"
                : text;
    }
}
