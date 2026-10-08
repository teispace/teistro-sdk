import static com.teispace.teistro.Civil.at;
import static com.teispace.teistro.Civil.date;
import static com.teispace.teistro.Civil.ianaZone;

import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;

import com.teispace.teistro.Altitude;
import com.teispace.teistro.Calendar;
import com.teispace.teistro.CalendarDate;
import com.teispace.teistro.Chart;
import com.teispace.teistro.ChartOptions;
import com.teispace.teistro.Context;
import com.teispace.teistro.ContextOptions;
import com.teispace.teistro.Ephemeris;
import com.teispace.teistro.IntlArea;
import com.teispace.teistro.Json;
import com.teispace.teistro.Latitude;
import com.teispace.teistro.Longitude;
import com.teispace.teistro.Observer;
import com.teispace.teistro.Teistro;
import com.teispace.teistro.TeistroException;
import com.teispace.teistro.ZoneResolution;
import com.teispace.teistro.blob.IntlRender;

/**
 * An interpretation: the same birth record, said in two languages.
 *
 * <p>{@code ChartReading.java} read the chart in full. This is what comes
 * after: a <b>composer</b> turns what was read into a narrative plan — an
 * ordered list of message keys and their slots — and the locale engine says
 * it. The plan holds no words at all, which is why one plan says the same
 * chart in English and in Nepali without the composer knowing either
 * language ({@code docs/03-design/plans-at-the-boundary.md}).
 *
 * <p>What it teaches:
 *
 * <ol>
 *   <li><b>The plan comes back in the same crossing as the chart.</b>
 *       {@code interpret} is an option on {@code found}, like {@code rules}
 *       and {@code vargas}; nothing is founded or evaluated twice, and a
 *       chart with no {@code interpret} pays nothing.
 *   <li><b>An item's {@code params} are {@code intl.render}'s own
 *       params.</b> There is no conversion step in this file, and there is
 *       none in the binding either: that is the whole design.
 *   <li><b>One plan, every locale.</b> The same plan is said twice below,
 *       and the rendering says which locale answered — so you can prove the
 *       locale had the message rather than quietly falling back to English.
 *   <li><b>A reading says what the rules found.</b> {@code readings} needs
 *       {@code rules} beside it, because it composes their answers rather
 *       than re-deriving them; asking for it alone is refused, by name.
 *   <li><b>A composer says what it can say.</b> The lagna stands in the
 *       chart and is in no placement item: those messages read a graha, and
 *       the lagna is a point. A composer that guessed would be worse than
 *       one that is quiet.
 * </ol>
 *
 * <p>The record is {@code BirthChart.java}'s own, so the examples can be
 * read side by side.
 */
public final class Interpretation {
    private Interpretation() {}

    /**
     * Runs the example.
     *
     * @param args unused
     */
    public static void main(String[] args) {
        Teistro teistro = Teistro.open();
        ContextOptions options = ContextOptions.builder()
                .profile("nepali-default")
                .locale("en-Latn")
                .ephemeris(Ephemeris.BUILTIN)
                .build();
        try (Context ctx = teistro.context(options)) {
            CalendarDate born = date(Calendar.BIKRAM_SAMBAT, 2042, 9, 17);
            ZoneResolution when = ctx.time().resolve(at(born, 0, 20), ianaZone("Asia/Kathmandu"));
            Observer place = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));

            // ── The chart, and what it has to say, in one call ─────────────
            Chart chart = ctx.chart().found(when.instantJdUtc(), place, when.offsetSeconds(), ChartOptions.builder()
                    // The rules whose answers the readings composer will say.
                    // The sections they read are computed whether or not they
                    // are asked for here.
                    .rules(Map.of("shipped", List.of("NABHASAS", "ARISHTAS")))
                    .interpret(Map.of(
                            "placements", true,
                            "readings", true,
                            "strength", true,
                            "houses", true,
                            "positions", true,
                            "aspects", true,
                            "conditions", true,
                            "karakas", true))
                    .build());
            Map<String, Object> plans = chart.plans().orElseThrow();
            List<Map<?, ?>> placements = items(plans, "placements");
            List<Map<?, ?>> readings = items(plans, "readings");
            List<Map<?, ?>> weights = items(plans, "strength");
            List<Map<?, ?>> ruled = items(plans, "houses");
            List<Map<?, ?>> degrees = items(plans, "positions");
            List<Map<?, ?>> looks = items(plans, "aspects");
            List<Map<?, ?>> states = items(plans, "conditions");
            List<Map<?, ?>> karakas = items(plans, "karakas");

            System.out.println("BS 2042-09-17  00:20  Kathmandu");
            System.out.println("plan     " + placements.size() + " placement items, "
                    + readings.size() + " reading items, " + weights.size() + " strengths, "
                    + ruled.size() + " lordships, " + degrees.size() + " positions, "
                    + looks.size() + " drishtis, " + states.size() + " conditions, "
                    + karakas.size() + " karakas");
            Set<String> keys = new LinkedHashSet<>();
            for (Map<?, ?> item : joined(placements, readings, weights, ruled, degrees, looks, states, karakas)) {
                keys.add(String.valueOf(item.get("key")));
            }
            System.out.println("keys     " + String.join(", ", keys));

            // ── The same plan, said twice ──────────────────────────────────
            // Nothing between an item and the renderer: `item.get("params")`
            // is what `render` takes, so this loop is the whole consumer story.
            for (String locale : List.of("en-Latn", "ne-Deva-NP")) {
                ctx.intl().setLocale(locale);
                System.out.println("\n" + locale);
                for (Map<?, ?> item : joined(placements, weights, ruled, degrees, looks, states, karakas)) {
                    IntlRender said = say(ctx.intl(), item);
                    System.out.println("  " + said.text() + (said.isFallback() != 0 ? "  (fallback)" : ""));
                }
                // A reading names its rule in a slot the message does not
                // print, so a consumer can group a plan by rule.
                for (Map<?, ?> item : readings) {
                    IntlRender said = say(ctx.intl(), item);
                    String fallback = said.isFallback() != 0 ? "  (fallback)" : "";
                    Map<?, ?> params = (Map<?, ?>) item.get("params");
                    System.out.println("  " + params.get("rule") + ": " + said.text() + fallback);
                }
            }

            // ── What it does not say, and what it refuses ──────────────────
            boolean lagna = placements.stream().anyMatch(item -> Json.write(item.get("params")).contains("LAGNA"));
            System.out.println("\nthe lagna is in the placements: " + lagna);
            // The Shadbala says whether a graha reaches its required rupas; no
            // locale says it, so the plan does not either.
            boolean strong = weights.stream().anyMatch(item -> Json.write(item.get("params")).contains("strong"));
            System.out.println("a strength item claims \"strong\": " + strong);
            // A bhava knows its sign, its cusps and its class; no locale says
            // any of them, so the houses plan says the lord and stops there.
            boolean signed = ruled.stream().anyMatch(item -> Json.write(item.get("params")).contains("rashi"));
            System.out.println("a houses item claims a sign: " + signed);
            // A position crosses as a number: the degree signs a reader sees
            // are the locale's rendering, never a string the composer wrote.
            boolean angled = degrees.stream().anyMatch(item -> Json.write(item.get("params")).contains("°"));
            System.out.println("a position item carries a rendered angle: " + angled);

            try {
                ctx.chart().found(when.instantJdUtc(), place, when.offsetSeconds(), ChartOptions.builder()
                        .interpret(Map.of("readings", true))
                        .build());
            } catch (TeistroException error) {
                System.out.println("refused  " + error.field() + ": " + error.getMessage());
                System.out.println("hint     " + error.hint());
            }
        }
    }

    /** A plan's items, each an object of {@code key} and {@code params}. */
    private static List<Map<?, ?>> items(Map<String, Object> plans, String name) {
        if (!(plans.get(name) instanceof List<?> found)) {
            throw new IllegalStateException("no `" + name + "` plan");
        }
        List<Map<?, ?>> items = new ArrayList<>();
        for (Object item : found) {
            items.add((Map<?, ?>) item);
        }
        return items;
    }

    /** Several plans' items, one after another. */
    @SafeVarargs
    private static List<Map<?, ?>> joined(List<Map<?, ?>>... plans) {
        List<Map<?, ?>> all = new ArrayList<>();
        for (List<Map<?, ?>> plan : plans) {
            all.addAll(plan);
        }
        return all;
    }

    /** An item said in the context's locale: its params are what the renderer takes. */
    private static IntlRender say(IntlArea intl, Map<?, ?> item) {
        return intl.renderJson(String.valueOf(item.get("key")), Json.write(item.get("params")));
    }
}
