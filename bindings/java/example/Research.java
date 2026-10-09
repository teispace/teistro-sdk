import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.function.Predicate;

import com.teispace.teistro.Altitude;
import com.teispace.teistro.Context;
import com.teispace.teistro.ContextOptions;
import com.teispace.teistro.Ephemeris;
import com.teispace.teistro.Latitude;
import com.teispace.teistro.Longitude;
import com.teispace.teistro.Observer;
import com.teispace.teistro.ResearchBirth;
import com.teispace.teistro.ResearchCounts;
import com.teispace.teistro.ResearchTested;
import com.teispace.teistro.Teistro;

/**
 * A two-group study, as {@code 03-design/research.md} asks one to be run: the predicates named, the
 * labels and the test fixed, and the input hash published before any data are read.
 * {@code cargo xtask check-parity} runs this file beside the other bindings' and compares what it
 * prints.
 */
public final class Research {
    private Research() {}

    /**
     * Runs the example.
     *
     * @param args unused
     */
    public static void main(String[] args) {
        Teistro teistro = Teistro.open();
        ContextOptions options = ContextOptions.builder()
                .profile("nepali-default")
                .ephemeris(Ephemeris.BUILTIN)
                .build();
        try (Context ctx = teistro.context(options)) {
            // Forty-eight births at Kathmandu, two months and an hour apart, and the first sixteen
            // called the cases. The labels mean nothing, so a study that reads them honestly finds
            // nothing: that is what the corrections are for.
            Observer place = new Observer(new Longitude(85.324), new Latitude(27.7172), new Altitude(1400));
            List<ResearchBirth> births = new ArrayList<>();
            List<Integer> groups = new ArrayList<>();
            for (int i = 0; i < 48; i += 1) {
                births.add(new ResearchBirth(2444240.5 + 61.37 * i + (i % 24) / 24.0, place, 20_700));
                groups.add(i < 16 ? 1 : 0);
            }
            Map<String, Object> rules = Map.of("shipped", List.of("YOGAS"));
            Map<String, Object> design = Map.of("groups", groups);

            ResearchCounts table = ctx.research().counts(births, rules, design, null);
            System.out.println(table.rows().size() + " rules over " + births.size() + " births");

            ResearchTested tested = ctx.research().compare(births, rules, design, Map.of("seed", 2026,
                    "permutations", 999, "contrast", Map.of("kind", "CASE_VS_REST", "case", 1), "alpha", 0.05),
                    null);
            // The registration: the births, the rules, the labels and the test.
            System.out.println("registered as " + tested.provenance().inputHash());
            System.out.printf(Locale.ROOT, "%d permutations, none can say less than p = %.4f%n",
                    tested.permutations(), tested.resolution());

            System.out.println("under 0.05: " + under(tested, ResearchTested.UnderAlpha::raw) + " raw, "
                    + under(tested, ResearchTested.UnderAlpha::maxT) + " after max-T, "
                    + under(tested, ResearchTested.UnderAlpha::holm) + " after Holm");

            // The smallest raw p, the first such row on a tie, and what the family makes of it.
            ResearchTested.Row best = tested.rows().get(0);
            for (ResearchTested.Row row : tested.rows()) {
                if (row.p().value() < best.p().value()) {
                    best = row;
                }
            }
            ResearchCounts.Group rest = best.counts().get(0);
            ResearchCounts.Group cases = best.counts().get(1);
            System.out.printf(Locale.ROOT, "%s: %d of %d cases, %d of %d others, p %.3f, max-T %.3f%n",
                    best.predicate(), cases.present(), cases.present() + cases.absent(), rest.present(),
                    rest.present() + rest.absent(), best.p().value(), best.adjusted().maxT());
            if (best.effect() != null) {
                ResearchTested.Estimate d = best.effect().riskDifference();
                System.out.printf(Locale.ROOT, "difference %.3f (%.3f to %.3f)%n", d.estimate(), d.low(), d.high());
            }
        }
    }

    private static long under(ResearchTested tested, Predicate<ResearchTested.UnderAlpha> method) {
        return tested.rows().stream().filter(row -> row.underAlpha() != null && method.test(row.underAlpha()))
                .count();
    }
}
