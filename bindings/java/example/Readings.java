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
import com.teispace.teistro.Teistro;
import com.teispace.teistro.blob.IntlRender;

/**
 * A rule's own reading, in the reader's language — loaded, not embedded.
 *
 * <p>{@code Interpretation.java} composed a plan and said it in two
 * languages. One thing it could not say in Nepali was <b>what a rule's verse
 * states</b>: that crosses as the words the rule cites, in the language the
 * rule was written in, and a Nepali reading shows the seam.
 *
 * <p>This is how the seam closes. The SDK carries a corpus of readings —
 * one for each of 649 yogas and doshas, in Sanskrit, Nepali, English and
 * Hindi — and it is <b>not compiled into the library</b>: it is several
 * times the size of every message pack together, and a consumer computing a
 * Julian day should not carry every Nepali yoga reading to do it. It is a
 * pack that is loaded ({@code docs/03-design/interpretation-records.md}).
 *
 * <p>What it teaches:
 *
 * <ol>
 *   <li><b>A pack is bytes.</b> {@code intl().loadPack} takes them from
 *       wherever you got them — a file beside your program, a download, an
 *       asset in your application bundle. This example reads the files
 *       {@code teistro-intl build} wrote from the SDK's own source root, as
 *       every binding's does.
 *   <li><b>Loading changes what a composer says</b>, not how it is called.
 *       {@code readings} asks the base locale whether it carries a reading
 *       of each rule: with the pack, the item is the locale's own reading;
 *       without it, the verse's cited words. The same code composes both.
 *   <li><b>A plan item is a sentence.</b> The record also holds the full
 *       passage and its named facets; those are read from the entity
 *       directly, which is one call on an engine you already have.
 * </ol>
 *
 * <pre>
 * cargo run -p teistro-intl -- --root packs/readings build --out target/packs/readings
 * </pre>
 */
public final class Readings {
    private Readings() {}

    /**
     * Where the built packs are: {@code TEISTRO_PACKS}, or the repository's
     * {@code target/packs}, from {@code bindings/java} where the example runs.
     */
    private static final Path PACKS = Path.of(
            System.getenv("TEISTRO_PACKS") != null ? System.getenv("TEISTRO_PACKS") : "../../target/packs");

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
            // ── The pack ─────────────────────────────────────────────────
            // One pack a locale, because a Nepali application wants Nepali
            // and its fallback and not four languages' worth of prose.
            for (Path file : packsOf("readings")) {
                byte[] data = Files.readAllBytes(file);
                // This is the call a consumer makes, whatever the bytes came
                // from.
                IntlLoaded loaded = ctx.intl().loadPack(data);
                System.out.printf(Locale.ROOT, "loaded %-12s %6d readings, %7d bytes%n",
                        loaded.locale(), loaded.entries(), data.length);
            }

            // ── A chart, and the rules it holds ──────────────────────────
            // The computed yogas are the set whose rules the corpus wrote
            // readings for; the nabhasas are the kernel's own and have none,
            // which is the gap `interpret-measured.md` counts.
            Observer kathmandu = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
            Chart chart = ctx.chart().found(2447995.4895833335, kathmandu, 20700, ChartOptions.builder()
                    .rules(Map.of("shipped", List.of("YOGAS", "DOSHAS")))
                    .interpret(Map.of("readings", true))
                    .build());
            if (!(chart.plans().orElseThrow().get("readings") instanceof List<?> plan)) {
                throw new IllegalStateException("no `readings` plan");
            }

            // ── The plan, said twice ─────────────────────────────────────
            System.out.println("\n" + plan.size() + " items");
            for (String locale : List.of("en-Latn", "ne-Deva-NP")) {
                ctx.intl().setLocale(locale);
                System.out.println("\n" + locale);
                for (Object item : plan) {
                    IntlRender said = say(ctx.intl(), (Map<?, ?>) item);
                    // A fallback would mean this locale had no reading of its
                    // own, which is exactly what an example must not hide.
                    String fallback = said.isFallback() != 0 ? "  (fallback)" : "";
                    System.out.println("  " + said.text() + fallback);
                }
            }

            // ── The passage, which the plan does not carry ───────────────
            // A plan item is a sentence. The record holds the essay and its
            // named facets beside it, for a page that wants them.
            Object reading = null;
            for (Object item : plan) {
                Map<?, ?> says = (Map<?, ?>) item;
                if ("sdk.reading.says".equals(says.get("key"))) {
                    reading = ((Map<?, ?>) says.get("params")).get("reading");
                    break;
                }
            }
            if (reading instanceof Map<?, ?> entity && entity.get("$entity") instanceof String key) {
                Map<String, String> forms = ctx.intl().entity(key).forms();
                System.out.println("\n" + key);
                for (String form : List.of("prose", "career", "mind", "spirituality")) {
                    if (forms.containsKey(form)) {
                        System.out.printf(Locale.ROOT, "  %-14s %s%n", form, firstSentence(forms.get(form)));
                    }
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

    /** An item said in the context's locale: its params are what the renderer takes. */
    private static IntlRender say(IntlArea intl, Map<?, ?> item) {
        return intl.renderJson(String.valueOf(item.get("key")), Json.write(item.get("params")));
    }

    /**
     * Enough of a passage to show it is there, without printing an essay; in
     * characters, not UTF-16 units, so a Devanagari passage is cut where
     * every binding cuts it.
     */
    private static String firstSentence(String text) {
        return text.codePointCount(0, text.length()) > 96
                ? text.substring(0, text.offsetByCodePoints(0, 96)) + "…"
                : text;
    }
}
