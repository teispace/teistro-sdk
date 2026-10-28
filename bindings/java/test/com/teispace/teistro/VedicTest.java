package com.teispace.teistro;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;

import com.teispace.teistro.request.MatchingRequest;
import com.teispace.teistro.request.SadeSatiRequest;
import com.teispace.teistro.request.ThemePatch;
import com.teispace.teistro.request.VarshaRequest;

/**
 * The Vedic readings the Node, Python and Dart suites hold and this one did
 * not: a chart's Sade Sati, each period whole, asked of a batch and refused
 * by the field it names; the muhurta search; the match with a partner, its
 * ten considerations, Kuja dosha and marriage doshas; the sahams of a
 * birth and of its years; and a chart drawn in every shipped layout.
 */
final class VedicTest {
    private VedicTest() {}

    private static void check(boolean holds, String what) {
        if (!holds) {
            throw new AssertionError(what);
        }
    }

    private static <T> void same(T expected, T found, String what) {
        if (!java.util.Objects.equals(expected, found)) {
            throw new AssertionError(what + ": expected " + expected + ", found " + found);
        }
    }

    private static final Observer KATHMANDU = new Observer(new Longitude(85.324), new Latitude(27.7172),
            new Altitude(1400));

    /**
     * Adds the tests to {@code tests}.
     *
     * @param tests the run's tests, by name
     * @param teistro the library
     */
    static void add(Map<String, BindingTest.Test> tests, Teistro teistro) {
        tests.put("an almanac carries the muhurta search it was asked for, each clause read as written", () -> {
            try (Context sky = teistro.context(ContextOptions.builder().ephemeris(Ephemeris.BUILTIN).build())) {
                CalendarDate from = Civil.date(Calendar.GREGORIAN, 2026, 11, 25);
                CalendarDate to = Civil.date(Calendar.GREGORIAN, 2026, 12, 3);
                java.util.function.BiFunction<CalendarDate, Map<String, Object>, Almanac> days = (last, asked) ->
                        sky.almanac().of(from, last, KATHMANDU, 20_700,
                                asked == null ? null : com.teispace.teistro.request.MuhurtaRequest.of(asked), null,
                                false, false, false);
                Almanac plain = days.apply(to, null);
                check(plain.muhurta().isEmpty(), "none unless asked");
                Almanac almanac = days.apply(to, Map.of("rules", "RAMAN_MARRIAGE", "native",
                        Map.of("star", "ROHINI", "moonSign", "TAURUS", "lagna", "LEO"), "daysWithWindows", 9,
                        "most", 1000));
                MuhurtaAnswer answer = almanac.muhurta().orElseThrow();
                check(!answer.windows().isEmpty(), "windows found");
                same("TEXTS", answer.ranking(), "ranked by the texts");
                for (int k = 0; k < plain.size(); k += 1) {
                    same(plain.at(k).provenance().contentHash(), almanac.at(k).provenance().contentHash(),
                            "the days are the ones asked without a search");
                }
                java.util.Set<String> clauses = new java.util.TreeSet<>();
                MuhurtaClauseKind amrit = null;
                for (MuhurtaWindow window : answer.windows()) {
                    check(window.at().fromJd() < window.at().toJd(), "a window has length");
                    for (MuhurtaClause clause : window.clauses()) {
                        MuhurtaClauseKind kind = clause.kind();
                        check(MuhurtaClauseKind.CLAUSES.contains(kind.clause()), "a clause the build names");
                        same(kind, MuhurtaClauseKind.read(Json.read(Json.write(kind.request()))),
                                kind.clause() + " read back as it is written");
                        clauses.add(kind.clause());
                        if (kind instanceof MuhurtaClauseKind.ChoghadiyaClause chosen
                                && chosen.choghadiya() == Choghadiya.AMRIT) {
                            amrit = kind;
                        }
                    }
                }
                check(clauses.contains("TARABALA"), "the native is read: " + clauses);
                check(clauses.size() >= 8, "the clauses a marriage weighs: " + clauses);
                java.util.Set<String> knobs = new java.util.HashSet<>();
                answer.provenance().appliedConventions().forEach(convention -> knobs.add(convention.knob()));
                check(knobs.containsAll(List.of("muhurta.asta", "muhurta.zodiacAt")), "the knobs it read: " + knobs);

                // A clause answered is a bar a request may name, as it was read.
                check(amrit != null, "an amrit choghadiya in the windows");
                Map<String, Object> graded = Map.of("best", List.of(), "middling", List.of(), "rejected", List.of(),
                        "otherwise", "MIDDLING");
                Map<String, Object> rules = new java.util.LinkedHashMap<>();
                rules.put("day", Map.of("tithis", graded, "nakshatras", graded, "yogas", graded, "karanas", graded,
                        "varas", graded, "chandrabala", Map.of("avoid", List.of())));
                rules.put("months", Map.of("reckoning", "ANY"));
                rules.put("lagnas", graded);
                rules.put("padas", List.of());
                rules.put("heeds", List.of());
                rules.put("bars", List.of(amrit.request()));
                rules.put("unjudged", List.of());
                rules.put("baseline", null);
                Map<String, Object> asked = new java.util.LinkedHashMap<>();
                asked.put("rules", rules);
                asked.put("daysWithWindows", 1);
                asked.put("most", 100_000);
                MuhurtaAnswer barred = days.apply(from, asked).muhurta().orElseThrow();
                List<MuhurtaWindow> struck = barred.windows().stream().filter(one -> !one.barredBy().isEmpty())
                        .toList();
                check(!struck.isEmpty(), "the bar read back strikes the windows it names");
                for (MuhurtaWindow window : struck) {
                    same(List.of(amrit), window.barredBy().stream().map(MuhurtaBar::clause).toList(), "by the bar");
                }
            }
        });

        tests.put("a chart carries its Sade Sati, each period whole", () -> {
            try (Context sky = teistro.context(ContextOptions.builder().profile("nepali-default")
                    .ephemeris(Ephemeris.BUILTIN).build())) {
                double[] births = {2_447_995.489_583_333_5, 2_451_545.2};
                java.util.function.BiFunction<Double, SadeSatiRequest, SadeSatiReport> found = (instant, asked) -> {
                    ChartOptions.Builder options = ChartOptions.builder();
                    if (asked != null) {
                        options.sadeSati(asked);
                    }
                    return sky.chart().found(instant, KATHMANDU, 20_700, options.build()).sadeSati().orElse(null);
                };
                same(null, found.apply(births[0], null), "none unless asked");
                for (Reckoning reckoning : Reckoning.values()) {
                    if (reckoning.id() < 0) {
                        continue;
                    }
                    SadeSatiRequest asked = SadeSatiRequest.builder().from(2_460_676.5).to(2_464_329)
                            .reckoning(reckoning).spells(List.of(4, 7, 8)).build();
                    ChartBatch batch = sky.chart().foundMany(births, KATHMANDU, 20_700,
                            ChartOptions.builder().sadeSati(asked).build());
                    for (int k = 0; k < births.length; k += 1) {
                        SadeSatiReport report = batch.at(k).sadeSati().orElseThrow();
                        same(found.apply(births[k], asked), report, "a batch is each alone");
                        same(reckoning, report.reckoning(), "the reckoning asked");
                        same(GocharFrom.MOON, report.reference().from(), "counted from the Moon");
                        check(report.sadeSati().size() + report.spells().size() > 0, "something in the window");
                        for (SadeSati one : report.sadeSati()) {
                            same(List.of(12, 1, 2), one.phases().stream().map(SadeSatiSpell::house).toList(),
                                    "the twelfth, the first and the second");
                            List<SadeSatiVisit> visits = new ArrayList<>();
                            one.phases().forEach(spell -> visits.addAll(spell.visits()));
                            visits.sort(Comparator.comparing(SadeSatiVisit::from));
                            for (int i = 1; i < visits.size(); i += 1) {
                                check(visits.get(i - 1).to() <= visits.get(i).from(), "the visits do not overlap");
                            }
                            // Asked at one instant inside its peak, from what the
                            // report named: the same Sade Sati, whole.
                            SadeSatiVisit peak = one.phases().get(1).visits().get(0);
                            SadeSatiReport now = found.apply(births[k], SadeSatiRequest.builder()
                                    .from((peak.from() + peak.to()) / 2).countedFrom(report.reference().from())
                                    .reckoning(report.reckoning()).build());
                            same(List.of(one), now.sadeSati(), reckoning + ": whole from its peak");
                        }
                        for (SadeSatiSpell spell : report.spells()) {
                            check(List.of(4, 7, 8).contains(spell.house()), "a spell asked for: " + spell.house());
                        }
                    }
                }
                Map<String, SadeSatiRequest> bad = Map.of(
                        "sadeSati.to", SadeSatiRequest.builder().from(2_460_676.5).to(2_460_600.5).build(),
                        "sadeSati.spells", SadeSatiRequest.builder().from(2_460_676.5).spells(List.of(2)).build());
                for (var refused : bad.entrySet()) {
                    try {
                        found.apply(births[0], refused.getValue());
                        throw new AssertionError(refused.getKey() + " was not refused");
                    } catch (TeistroException e) {
                        same(refused.getKey(), e.field(), "refused by name");
                    }
                }
            }
        });
        tests.put("a chart is drawn in every shipped layout, each sign and house once a ring, every point on the page", () -> {
            try (Context sky = teistro.context(ContextOptions.builder().profile("nepali-default")
                    .ephemeris(Ephemeris.BUILTIN).build())) {
                List<ChartLayout> layouts = java.util.Arrays.stream(ChartLayout.values())
                        .filter(layout -> layout != ChartLayout.UNKNOWN).toList();
                java.util.function.Function<ThemePatch, List<Drawing>> drawn = theme -> {
                    ChartOptions.Builder asked = ChartOptions.builder().theme(theme);
                    for (ChartLayout layout : layouts) {
                        asked.drawing(layout, Varga.D1);
                    }
                    asked.drawing(ChartLayout.NORTH_INDIAN, Varga.D9);
                    return sky.chart().found(2_451_545.0, KATHMANDU, 20_700, asked.build()).drawings();
                };
                java.util.function.Consumer<UnitPoint> onPage = point -> check(
                        point.x() >= -1e-9 && point.x() <= 1 + 1e-9 && point.y() >= -1e-9 && point.y() <= 1 + 1e-9,
                        "a point on the unit page: " + point);
                java.util.function.Consumer<Outline> traced = outline -> {
                    onPage.accept(outline.start());
                    for (Segment step : outline.segments()) {
                        switch (step) {
                            case LineSegment line -> onPage.accept(line.to());
                            case QuadSegment quad -> {
                                onPage.accept(quad.control());
                                onPage.accept(quad.to());
                            }
                            case ArcSegment arc -> {
                                onPage.accept(arc.centre());
                                onPage.accept(arc.to());
                            }
                        }
                    }
                };
                List<Drawing> plain = drawn.apply(null);
                same(layouts.size() + 1, plain.size(), "a drawing for each asked");
                java.util.Set<String> steps = new java.util.TreeSet<>();
                for (int k = 0; k < plain.size(); k += 1) {
                    Drawing drawing = plain.get(k);
                    same(k < layouts.size() ? layouts.get(k) : ChartLayout.NORTH_INDIAN, drawing.layout(),
                            "in the order asked");
                    same(k < layouts.size() ? Varga.D1 : Varga.D9, drawing.varga(), "the varga asked");
                    same(null, drawing.svg(), "no theme, no SVG");
                    Map<Integer, List<DrawnCell>> rings = new java.util.TreeMap<>();
                    for (DrawnCell cell : drawing.cells()) {
                        rings.computeIfAbsent(cell.ring(), ring -> new ArrayList<>()).add(cell);
                        traced.accept(cell.outline());
                        onPage.accept(cell.label());
                        onPage.accept(cell.anchor());
                        cell.outline().segments().forEach(step -> steps.add(step.getClass().getSimpleName()));
                    }
                    check(!rings.isEmpty(), drawing.layout() + " has cells");
                    // The inner ring counts its houses from the lagna; an outer one (the
                    // Sudarshan chakra's Moon and Sun) from its own, still marking the lagna's sign.
                    Rashi lagna = rings.values().iterator().next().stream().filter(cell -> cell.house() == 1)
                            .findFirst().orElseThrow().sign();
                    for (var ring : rings.entrySet()) {
                        List<DrawnCell> cells = ring.getValue();
                        String where = drawing.layout() + " ring " + ring.getKey();
                        same(12, cells.size(), where + ": twelve cells");
                        same(12L, cells.stream().map(DrawnCell::house).distinct().count(), where + ": each house once");
                        // A wheel's cells are its houses, so a sign may hold two cusps and
                        // another none; a square chart's are its signs.
                        boolean wheel = drawing.layout() == ChartLayout.WESTERN_WHEEL;
                        if (!wheel) {
                            same(12L, cells.stream().map(DrawnCell::sign).distinct().count(), where + ": each sign once");
                        }
                        for (DrawnCell cell : cells) {
                            same(wheel ? cell.house() == 1 : cell.sign() == lagna, cell.lagna(),
                                    where + ": the lagna marked");
                        }
                    }
                    drawing.frame().forEach(traced);
                    for (DrawnMark mark : drawing.marks()) {
                        onPage.accept(mark.at());
                        check(mark.longitudeDeg() >= 0 && mark.longitudeDeg() < 360, "a mark's longitude");
                        check(!mark.body().isEmpty(), "a mark names its body");
                    }
                }
                check(steps.contains("LineSegment"), "outlines drawn: " + steps);

                List<Drawing> themed = drawn.apply(ThemePatch.builder().extendsValue("DARK").build());
                for (int k = 0; k < themed.size(); k += 1) {
                    String svg = themed.get(k).svg();
                    check(svg != null && svg.startsWith("<svg xmlns=\"http://www.w3.org/2000/svg\""),
                            "an SVG for each: " + themed.get(k).layout());
                    same(plain.get(k).cells(), themed.get(k).cells(), "a theme moves no cell");
                }
                check(themed.get(0).svg().contains("fill=\"#121212\""), "the dark page");
            }
        });

        tests.put("a muhurta search closes a day by its blackouts, and the baseline ranks by score", () -> {
            try (Context sky = teistro.context(ContextOptions.builder().ephemeris(Ephemeris.BUILTIN).build())) {
                // 1 June 2026 is in Jyeshtha's adhika month.
                CalendarDate june = Civil.date(Calendar.GREGORIAN, 2026, 6, 1);
                MuhurtaAnswer closed = sky.almanac().of(june, june, KATHMANDU, 20_700,
                        com.teispace.teistro.request.MuhurtaRequest.of(Map.of("rules", "RAMAN_MARRIAGE")), null,
                        false, false, false).muhurta().orElseThrow();
                same(1, closed.closed().size(), "the one day closed");
                ClosedDay day = closed.closed().get(0);
                same(List.of(2026, 6, 1), List.of(day.date().year(), day.date().month(), day.date().day()),
                        "the day asked");
                check(day.by().contains(BlackoutKind.ADHIKA_MASA), "closed by the adhika month: " + day.by());
                check(closed.windows().isEmpty(), "no window on a closed day");

                CalendarDate from = Civil.date(Calendar.GREGORIAN, 2026, 11, 25);
                CalendarDate to = Civil.date(Calendar.GREGORIAN, 2026, 12, 3);
                MuhurtaAnswer scored = sky.almanac().of(from, to, KATHMANDU, 20_700,
                        com.teispace.teistro.request.MuhurtaRequest.of(Map.of("rules", "BASELINE_MARRIAGE",
                                "ranking", "BASELINE", "most", 1000)), null, false, false, false)
                        .muhurta().orElseThrow();
                same("BASELINE", scored.ranking(), "ranked by the baseline");
                check(!scored.windows().isEmpty(), "windows found");
                // Open windows first, then those the engine would have offered by their
                // score, highest first; a window it would not have offered has none.
                Integer last = null;
                int open = 0;
                int scoredWindows = 0;
                for (MuhurtaWindow window : scored.windows()) {
                    if (window.barredBy().isEmpty()) {
                        open += 1;
                    }
                    MuhurtaScore score = window.score();
                    if (score == null) {
                        continue;
                    }
                    scoredWindows += 1;
                    for (MuhurtaFactor factor : score.factors()) {
                        check(!factor.dimension().isEmpty(), "a factor names its dimension");
                    }
                    if (window.barredBy().isEmpty()) {
                        check(last == null || score.value() <= last, "highest first: " + score.value() + " after "
                                + last);
                        last = score.value();
                    }
                }
                check(scoredWindows > 0, "the engine's scores read");
                for (int k = 0; k < open; k += 1) {
                    check(scored.windows().get(k).barredBy().isEmpty(), "the open windows first");
                }
            }
        });

        tests.put("a chart carries its match with a partner: the ten considerations, Kuja and the doshas", () -> {
            double birth = 2_451_545.0;
            Map<String, Object> partner = Map.of("instant", birth, "utcOffsetSeconds", 20_700,
                    "place", Map.of("latitude", 27.7172, "longitude", 85.324, "altitude", 1400.0));
            try (Context sky = teistro.context(ContextOptions.builder().profile("nepali-default")
                    .ephemeris(Ephemeris.BUILTIN).build())) {
                java.util.function.BiFunction<Double, MatchingRequest, Chart> found = (instant, asked) ->
                        sky.chart().found(instant, KATHMANDU, 20_700,
                                ChartOptions.builder().matching(asked).build());
                Chart plain = found.apply(birth, null);
                check(plain.porutham().isEmpty() && plain.kuja().isEmpty() && plain.marriageDoshas().isEmpty(),
                        "none unless asked");
                Chart self = found.apply(birth, MatchingRequest.of(Map.of("partner", partner, "partnerRole", "BRIDE",
                        "kuja", Map.of("from", "LAGNA_MOON_VENUS"))));

                // One star in one sign shares its Rajju, which the one lord lifts.
                Porutham ten = self.porutham().orElseThrow();
                same(List.of(Koota.TARA, Koota.GANA, Koota.MAHENDRA, Koota.STREE_DEERGHA, Koota.YONI, Koota.BHAKOOT,
                        Koota.GRAHA_MAITRI, Koota.VASHYA, Koota.RAJJU, Koota.VEDHA),
                        ten.considerations().stream().map(row -> row.reading().koota()).toList(),
                        "the ten in the source's order");
                DhinamPorutham dhinam = (DhinamPorutham) ten.considerations().get(0).reading();
                same(1, dhinam.count(), "one star counts one");
                check(dhinam.rule().name().startsWith("COMMON_"), "the common rule: " + dhinam.rule());
                PoruthamRow rajju = ten.considerations().get(8);
                RajjuPorutham divisions = (RajjuPorutham) rajju.reading();
                same(List.of(divisions.bride(), true, true),
                        List.of(divisions.groom(), rajju.agrees(), rajju.lifted()), "one Rajju, lifted");
                same(List.of(true, false), List.of(ten.exception().oneLord(), ten.exception().opposite()),
                        "the one lord, not opposite");
                same((int) ten.considerations().stream().filter(PoruthamRow::agrees).count(), ten.agreeing(),
                        "the agreeing counted");

                // Manasagari: one birth on both sides reads Mars alike.
                Kuja mars = self.kuja().orElseThrow();
                same(mars.bride(), mars.groom(), "both sides alike");
                same(List.of("LAGNA", "MOON", "VENUS"),
                        mars.bride().readings().stream().map(KujaReading::reference).toList(), "from the three");
                for (KujaReading reading : mars.bride().readings()) {
                    same(List.of(1, 4, 7, 8, 12).contains(reading.house()), reading.inHouses(),
                            reading.reference() + " in the houses");
                }
                same(mars.bride().readings().stream().anyMatch(KujaReading::inHouses), mars.bride().dosha(),
                        "a dosha where one stands in them");
                same(mars.bride().dosha(), mars.both(), "both, or neither");

                // C289: the shared nadi first, then each consideration that disagrees or was
                // lifted, then the Kuja sides.
                List<MarriageDosha> doshas = self.marriageDoshas().orElseThrow();
                same(new MarriageDosha(DoshaSystem.ASHTA_KOOTA, Koota.NADI, null, false), doshas.get(0),
                        "the shared nadi first");
                List<String> porutham = new ArrayList<>();
                List<MatchRole> sides = new ArrayList<>();
                for (MarriageDosha dosha : doshas) {
                    if (dosha.system() == DoshaSystem.PORUTHAM) {
                        porutham.add(dosha.koota() + " " + dosha.lifted());
                        same(null, dosha.side(), "a consideration has no side");
                    } else if (dosha.system() == DoshaSystem.KUJA) {
                        sides.add(dosha.side());
                        same(null, dosha.koota(), "a Kuja side has no koota");
                    }
                }
                List<String> expected = new ArrayList<>();
                for (PoruthamRow row : ten.considerations()) {
                    if (!row.agrees() || row.lifted()) {
                        expected.add(row.reading().koota() + " " + row.lifted());
                    }
                }
                same(expected, porutham, "each consideration that disagrees or was lifted");
                same(mars.bride().dosha() ? List.of(MatchRole.BRIDE, MatchRole.GROOM) : List.of(), sides,
                        "the Kuja sides");

                // A batch reads each chart as the chart alone reads it.
                MatchingRequest asked = MatchingRequest.of(Map.of("partner",
                        Map.of("instant", 2_447_892.5, "utcOffsetSeconds", 20_700,
                                "place", Map.of("latitude", 27.7172, "longitude", 85.324, "altitude", 1400.0)),
                        "partnerRole", "GROOM", "kuja", Map.of()));
                double[] instants = {birth, birth + 9.5, birth + 17.25};
                ChartBatch batch = sky.chart().foundMany(instants, KATHMANDU, 20_700,
                        ChartOptions.builder().matching(asked).build());
                for (int k = 0; k < instants.length; k += 1) {
                    Chart one = found.apply(instants[k], asked);
                    Chart many = batch.at(k);
                    same(one.porutham(), many.porutham(), "porutham " + k);
                    same(one.kuja(), many.kuja(), "kuja " + k);
                    same(one.marriageDoshas(), many.marriageDoshas(), "doshas " + k);
                }
            }
        });

        tests.put("a birth and each of its years carry the sahams asked for, each with its strength", () -> {
            double instant = 2_447_995.4895833335;
            try (Context sky = teistro.context(ContextOptions.builder().profile("nepali-default")
                    .ephemeris(Ephemeris.BUILTIN).build())) {
                java.util.function.Function<Map<String, Object>, Chart> found = varsha ->
                        sky.chart().found(instant, KATHMANDU, 20_700,
                                ChartOptions.builder().varsha(VarshaRequest.of(varsha)).build());
                Chart chart = found.apply(Map.of("through", 2, "place", "birth", "sahams", "all"));
                same(41, chart.sahams().size(), "the birth's own forty-one");
                for (TajikaSaham saham : chart.sahams()) {
                    check(!saham.strong().contains(SahamStrong.WITH_YEAR_LORD), "a birth has no year lord");
                }
                for (Pravesha year : chart.praveshas()) {
                    AnnualChart annual = year.annual();
                    same(41, annual.sahams().size(), "each year's forty-one");
                    int lagna = (int) (annual.lagnaDeg() / 30);
                    for (TajikaSaham saham : annual.sahams()) {
                        int sign = (int) (saham.longitudeDeg() / 30);
                        same(Rashi.of(sign), saham.sign(), saham.saham() + " in the sign of its longitude");
                        same(Math.floorMod(sign - lagna, 12) + 1, saham.house(), saham.saham() + " in its house");
                        boolean near = saham.strong().contains(SahamStrong.LORD_CONJOINS)
                                || saham.strong().contains(SahamStrong.LORD_ASPECTS_SAHAM);
                        // The two (c) clauses negate each other: exactly one holds.
                        same(!near, saham.weak().contains(SahamWeak.LORD_APART), saham.saham() + " near or apart");
                        same(7, saham.seven().size(), "seven grahas under it");
                        same(saham.strong().contains(SahamStrong.LORD_CONJOINS),
                                saham.seven().stream().filter(s -> s.graha() == saham.lord()).findFirst()
                                        .orElseThrow().company(),
                                saham.saham() + ": its lord's company");
                        check(saham.inNodeAxis() != null, "the nodes read");
                    }
                }
                Chart natal = found.apply(Map.of("through", 1, "sahams", List.of("PUNYA")));
                same(List.of(Saham.PUNYA), natal.sahams().stream().map(TajikaSaham::saham).toList(),
                        "only the one asked");
                same(null, natal.praveshas().get(0).annual(), "no year without a place");
                check(found.apply(Map.of("through", 1, "place", "birth")).sahams().isEmpty(), "none unless asked");
                try {
                    found.apply(Map.of("through", 2, "place", "birth", "sahams", List.of("PUNYA", "PUNYA")));
                    throw new AssertionError("a saham asked twice was not refused");
                } catch (TeistroException e) {
                    same("varsha.sahams", e.field(), "refused by name");
                }
            }
        });
    }
}
