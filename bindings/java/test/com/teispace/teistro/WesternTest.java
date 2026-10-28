package com.teispace.teistro;

import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.function.Supplier;

import com.teispace.teistro.request.FortitudeRequest;
import com.teispace.teistro.request.HarmonicRequest;
import com.teispace.teistro.request.ParallelRequest;
import com.teispace.teistro.request.PartnerSynastry;
import com.teispace.teistro.request.PerfectionRequest;

/**
 * The horary and Western readings the Node, Python and Dart suites hold and
 * this one did not: Lilly's considerations, a matter's perfection, a
 * harmonic chart, the declinations and a synastry with all it carries,
 * each read whole, asked of a batch, and refused by the field it names.
 */
final class WesternTest {
    private WesternTest() {}

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

    private static String refusedField(Runnable call) {
        try {
            call.run();
        } catch (TeistroException e) {
            return e.field();
        }
        throw new AssertionError("the call was not refused");
    }

    private static final Observer LONDON = new Observer(new Longitude(-0.12), new Latitude(51.5), new Altitude(0));

    private static final List<Double> LILLYS_ORBS = List.of(10.0, 12.0, 7.5, 17.0, 8.0, 7.0, 12.5);

    private static Context baseline(Teistro teistro) {
        return teistro.context(ContextOptions.builder().profile("conformance-baseline").ephemeris(Ephemeris.BUILTIN)
                .build());
    }

    private static Context tropical(Teistro teistro) {
        return teistro.context(ContextOptions.builder().profile("western-tropical-default")
                .ephemeris(Ephemeris.BUILTIN).build());
    }

    /** An answer's rules, as the request they stand for. */
    private static com.teispace.teistro.request.ConsiderationRules asked(ConsiderationRules rules) {
        return com.teispace.teistro.request.ConsiderationRules.builder().moonLateFromDeg(rules.moonLateFromDeg())
                .orbsDeg(rules.orbsDeg()).build();
    }

    /** An answer's rules, as the request they stand for. */
    private static com.teispace.teistro.request.PerfectionRules asked(PerfectionRules rules) {
        var building = com.teispace.teistro.request.PerfectionRules.builder().orbsDeg(rules.orbsDeg())
                .withinSign(rules.withinSign());
        if (rules.horizonDays() != null) {
            building.horizonDays(rules.horizonDays());
        }
        return building.build();
    }

    /**
     * Adds the tests to {@code tests}.
     *
     * @param tests the run's tests, by name
     * @param teistro the library
     */
    static void add(Map<String, BindingTest.Test> tests, Teistro teistro) {
        tests.put("a chart carries its considerations", () -> {
            try (Context sky = baseline(teistro)) {
                double[] instants = new double[16];
                for (int k = 0; k < instants.length; k += 1) {
                    instants[k] = 2_451_545 + k / 8.0;
                }
                com.teispace.teistro.request.ConsiderationRules lilly =
                        com.teispace.teistro.request.ConsiderationRules.builder().build();
                check(sky.chart().found(instants[0], LONDON, 0, ChartOptions.builder().build()).considerations()
                        .isEmpty(), "none unless asked");
                int voids = 0;
                for (double instant : instants) {
                    Chart chart = sky.chart().found(instant, LONDON, 0, ChartOptions.builder().considerations(lilly)
                            .fortitudes(new FortitudeRequest()).build());
                    Considerations read = chart.considerations().orElseThrow();
                    AccidentalSky heavens = chart.fortitudes().orElseThrow().sky();
                    same(new ConsiderationRules(27, LILLYS_ORBS), read.rules(), "Lilly's rules");
                    same((int) (heavens.ascendantDeg() / 30), read.ascendant().sign().id(), "the rising sign");
                    same(heavens.cuspsDeg().get(6), read.seventh().cuspDeg(), "the seventh cusp");
                    same(read.ascendant().degree() < 3, read.ascendant().early(), "too early");
                    MoonCourse course = read.moon().course();
                    if (course.next() == null) {
                        same(null, course.withinOrb(), "no contact within orb without a next");
                    }
                    if (course.next() == null || course.withinOrb() == null) {
                        voids += 1;
                    }
                    same(read.radicality().hourLord() == read.radicality().ascendantLord(),
                            read.radicality().grounds().contains(RadicalGround.ONE_LORD), "one lord");
                    same(read, sky.chart().found(instant, LONDON, 0, ChartOptions.builder()
                            .considerations(asked(read.rules())).build()).considerations().orElseThrow(),
                            "an answer's rules are a request as they stand");
                }
                check(voids > 0, "a void Moon in the batch");

                var wider = com.teispace.teistro.request.ConsiderationRules.builder().moonLateFromDeg(25)
                        .orbsDeg(List.of(9.0, 9.0, 7.0, 15.0, 7.0, 7.0, 12.0)).build();
                same(new ConsiderationRules(25, wider.orbsDeg()), sky.chart().found(instants[0], LONDON, 0,
                        ChartOptions.builder().considerations(wider).build()).considerations().orElseThrow().rules(),
                        "the rules asked");
                ChartBatch batch = sky.chart().foundMany(instants, LONDON, 0,
                        ChartOptions.builder().considerations(lilly).build());
                for (int k = 0; k < instants.length; k += 1) {
                    same(sky.chart().found(instants[k], LONDON, 0, ChartOptions.builder().considerations(lilly)
                            .build()).considerations(), batch.at(k).considerations(), "a batch, chart " + k);
                }
                Map<String, com.teispace.teistro.request.ConsiderationRules> bad = Map.of(
                        "considerations.moonLateFromDeg",
                        com.teispace.teistro.request.ConsiderationRules.builder().moonLateFromDeg(31).build(),
                        "considerations.orbsDeg", com.teispace.teistro.request.ConsiderationRules.builder()
                                .orbsDeg(List.of(10.0, 12.0, 7.5, 17.0, 8.0, 7.0, -1.0)).build());
                for (var refused : bad.entrySet()) {
                    same(refused.getKey(), refusedField(() -> sky.chart().found(instants[0], LONDON, 0,
                            ChartOptions.builder().considerations(refused.getValue()).build())), "refused by name");
                }
            }
        });

        tests.put("a chart carries its perfection", () -> {
            try (Context sky = baseline(teistro)) {
                double[] instants = new double[12];
                for (int k = 0; k < instants.length; k += 1) {
                    instants[k] = 2_451_545 + 23 * k + k / 7.0;
                }
                PerfectionRequest seventh = PerfectionRequest.builder().house(7).build();
                check(sky.chart().found(instants[0], LONDON, 0, ChartOptions.builder().build()).perfection()
                        .isEmpty(), "none unless asked");
                int applying = 0;
                int hindered = 0;
                for (double instant : instants) {
                    Chart chart = sky.chart().found(instant, LONDON, 0, ChartOptions.builder().perfection(seventh)
                            .fortitudes(new FortitudeRequest()).build());
                    Matter read = chart.perfection().orElseThrow();
                    same(new PerfectionRules(LILLYS_ORBS, null, true), read.rules(), "Lilly's rules");
                    check(read.querent() != read.quesited(), "two significators");
                    Map<Graha, Integer> houses = new HashMap<>();
                    for (PlanetAccidents at : chart.fortitudes().orElseThrow().planets()) {
                        houses.put(at.planet(), at.house());
                    }
                    same(houses.get(read.querent()), read.ways().querent().house(), "the querent's house");
                    same(houses.get(read.quesited()), read.ways().quesited().house(), "the quesited's house");
                    same(read.querent(), read.ways().querent().planet(), "the querent's place");
                    if (read.application() != null) {
                        applying += 1;
                        Application application = read.application();
                        check(application.days() >= 0 && application.days() <= read.horizonDays(), "in the horizon");
                        check(Set.of(read.querent(), read.quesited()).contains(application.applying()),
                                "a significator applies");
                    }
                    for (Way way : List.of(Way.CONJUNCTION, Way.SEXTILE_OR_TRINE, Way.SQUARE, Way.OPPOSITION)) {
                        if (read.ways().held().contains(way)) {
                            check(read.application() != null, way + " is an application");
                        }
                    }
                    for (Impediment impediment : read.impediments()) {
                        hindered += 1;
                        same(impediment.kind() == ImpedimentKind.REFRANATION, impediment.third() == null,
                                "only refranation has no third");
                    }
                    for (Translation translation : read.translations()) {
                        same(Set.of(read.querent(), read.quesited()), Set.of(translation.from(), translation.to()),
                                "light carried between the significators");
                    }
                    same(read, sky.chart().found(instant, LONDON, 0, ChartOptions.builder().perfection(
                            PerfectionRequest.builder().house(7).rules(asked(read.rules())).build()).build())
                            .perfection().orElseThrow(), "an answer's rules are a request as they stand");
                }
                check(applying > 0, "the sweep applies");
                check(hindered > 0, "and is hindered");

                Matter named = sky.chart().found(instants[0], LONDON, 0, ChartOptions.builder().perfection(
                        PerfectionRequest.builder().querent(Graha.VENUS).quesited(Graha.MARS)
                                .rules(com.teispace.teistro.request.PerfectionRules.builder().horizonDays(30).build())
                                .build()).build()).perfection().orElseThrow();
                same(List.of(Graha.VENUS, Graha.MARS, 30.0, 30.0), List.of(named.querent(), named.quesited(),
                        named.rules().horizonDays(), named.horizonDays()), "between two named, in thirty days");
                check(!sky.chart().found(instants[0], LONDON, 0, ChartOptions.builder().perfection(
                        PerfectionRequest.builder().querent(Graha.VENUS).quesited(Graha.MARS)
                                .rules(com.teispace.teistro.request.PerfectionRules.builder().withinSign(false)
                                        .build()).build()).build()).perfection().orElseThrow().rules().withinSign(),
                        "across signs");
                ChartBatch batch = sky.chart().foundMany(instants, LONDON, 0,
                        ChartOptions.builder().perfection(seventh).build());
                for (int k = 0; k < instants.length; k += 1) {
                    same(sky.chart().found(instants[k], LONDON, 0, ChartOptions.builder().perfection(seventh).build())
                            .perfection(), batch.at(k).perfection(), "a batch, chart " + k);
                }
                Map<String, Supplier<PerfectionRequest>> bad = Map.of(
                        "perfection.quesited", () -> PerfectionRequest.builder().build(),
                        "perfection.house", () -> PerfectionRequest.builder().house(7).quesited(Graha.MARS).build(),
                        "perfection.rules.horizonDays", () -> PerfectionRequest.builder().house(7)
                                .rules(com.teispace.teistro.request.PerfectionRules.builder().horizonDays(-1).build())
                                .build());
                for (var refused : bad.entrySet()) {
                    same(refused.getKey(), refusedField(() -> sky.chart().found(instants[0], LONDON, 0,
                            ChartOptions.builder().perfection(refused.getValue().get()).build())), "refused by name");
                }
            }
        });

        tests.put("a chart carries its harmonic", () -> {
            // Churchill's 9th harmonic as Addey reads it (Harmonics in
            // Astrology, pp. 97-98): the Moon on Saturn in the third, Venus
            // rising, Pluto in the tenth.
            try (Context sky = tropical(teistro)) {
                Observer blenheim = new Observer(new Longitude(-1.3611), new Latitude(51.8414), new Altitude(0));
                double birth = 2_405_857.564_892;
                check(sky.chart().found(birth, blenheim, 0, ChartOptions.builder().outerPlanets(true).build())
                        .harmonic().isEmpty(), "none unless asked");
                HarmonicChart ninth = sky.chart().found(birth, blenheim, 0, ChartOptions.builder().outerPlanets(true)
                        .harmonic(new HarmonicRequest(9)).build()).harmonic().orElseThrow();
                same(9, ninth.harmonic(), "the ninth");
                same(12, ninth.points().size(), "ten bodies and two angles");
                Map<HarmonicPoint, Integer> houses = new HashMap<>();
                for (HarmonicPlaced placed : ninth.points()) {
                    houses.put(placed.point(), placed.house());
                    check(placed.longitudeDeg() >= 0 && placed.longitudeDeg() < 360, "a longitude on the circle");
                }
                same(List.of(3, 3, 1, 10), List.of(houses.get(HarmonicPoint.graha(Graha.MOON)),
                        houses.get(HarmonicPoint.graha(Graha.SATURN)), houses.get(HarmonicPoint.graha(Graha.VENUS)),
                        houses.get(HarmonicPoint.graha(Graha.PLUTO))), "Addey's houses");
                same(1, houses.get(HarmonicPoint.ASCENDANT), "the ascendant rises");
                same(HarmonicPoint.MIDHEAVEN, ninth.points().get(ninth.points().size() - 1).point(), "the midheaven last");
                HarmonicRow row = ninth.rows().stream().filter(one -> one.first().equals(HarmonicPoint.graha(Graha.MOON))
                        && one.second().equals(HarmonicPoint.graha(Graha.SATURN))).findFirst().orElseThrow();
                check(row.apartDeg() < 0.6, "the Moon on Saturn: " + row.apartDeg());
                same(List.of(4, 12.0), List.of(row.multiple(), row.orbDeg()), "a square of the ninth, in orb");

                HarmonicRequest fifth = new HarmonicRequest(5, 3.0);
                double[] instants = {birth, birth + 100.5};
                ChartBatch batch = sky.chart().foundMany(instants, blenheim, 0, ChartOptions.builder()
                        .outerPlanets(true).harmonic(fifth).build());
                for (int k = 0; k < instants.length; k += 1) {
                    same(sky.chart().found(instants[k], blenheim, 0, ChartOptions.builder().outerPlanets(true)
                            .harmonic(fifth).build()).harmonic(), batch.at(k).harmonic(), "a batch, chart " + k);
                }
                same("harmonic.number", refusedField(() -> sky.chart().found(birth, blenheim, 0, ChartOptions
                        .builder().harmonic(new HarmonicRequest(0)).build())), "no zeroth harmonic");
                same("harmonic.orbDeg", refusedField(() -> sky.chart().found(birth, blenheim, 0, ChartOptions
                        .builder().harmonic(new HarmonicRequest(9, 31.0)).build())), "no orb past thirty");
            }
        });

        tests.put("a chart carries its declinations", () -> {
            // King George V (Leo, How to Judge a Nativity, p. 130).
            try (Context sky = tropical(teistro)) {
                Observer george = new Observer(new Longitude(-0.1366), new Latitude(51.5045), new Altitude(0));
                double birth = 2_402_390.554_166_667;
                check(sky.chart().found(birth, george, 0, ChartOptions.builder().build()).declinations().isEmpty(),
                        "none unless asked");
                Declinations read = sky.chart().found(birth, george, 0, ChartOptions.builder().outerPlanets(true)
                        .parallels(new ParallelRequest()).build()).declinations().orElseThrow();
                same(10, read.grahas().size(), "ten bodies");
                check(Math.abs(read.graha(Graha.SUN).orElseThrow() - 22.2997) < 0.01, "the Sun's declination");
                check(Math.abs(read.lagnaDeg() - 0.8366) < 0.01, "the ascendant's");
                check(Math.abs(read.midheavenDeg() + 23.452) < 0.01, "the midheaven's");
                check(read.obliquityDeg() > 23.4 && read.obliquityDeg() < 23.5, "the obliquity of 1865");
                same(java.util.Optional.empty(), sky.chart().found(birth, george, 0, ChartOptions.builder()
                        .parallels(new ParallelRequest()).build()).declinations().orElseThrow().graha(Graha.PLUTO),
                        "an outer planet not asked for");
            }
        });
        // King George V and Queen Mary (Leo, *How to Judge a Nativity*, p. 130),
        // against the SDK test's recast (`03-design/western-synastry.md`,
        // `03-design/western-composites.md`).
        tests.put("a chart carries its synastry with a partner, and all it reads across", () -> {
            Observer george = new Observer(new Longitude(-0.1366), new Latitude(51.5045), new Altitude(0));
            double birth = 2_402_390.554166667;
            Map<String, Object> mary = Map.of("instant", 2_403_113.499305556, "utcOffsetSeconds", 0,
                    "place", Map.of("latitude", 51.5058, "longitude", -0.1878, "altitude", 0.0));
            java.util.function.DoubleUnaryOperator turn = d -> Math.abs(((d + 540) % 360) - 180);
            try (Context sky = tropical(teistro)) {
                java.util.function.BiFunction<Double, Map<String, Object>, Chart> found = (instant, asked) ->
                        sky.chart().found(instant, george, 0, ChartOptions.builder().outerPlanets(true)
                                .synastry(asked == null ? null : PartnerSynastry.of(asked)).build());
                java.util.function.Function<Map<String, Object>, Chart> with = more -> {
                    Map<String, Object> asked = new HashMap<>(more);
                    asked.put("partner", mary);
                    return found.apply(birth, asked);
                };
                Chart none = found.apply(birth, null);
                check(none.synastry().isEmpty(), "no synastry unless asked");

                Chart plain = with.apply(Map.of());
                List<SynastryRow> rows = plain.synastry().orElseThrow();
                java.util.function.Function<NatalPoint, String> named = point ->
                        point.point() + (point.graha() == null ? "" : " " + point.graha());
                for (Object[] contact : new Object[][] {
                    {"GRAHA MARS", WesternAspect.OPPOSITION, "LAGNA", 0.32},
                    {"GRAHA MARS", WesternAspect.SEXTILE, "GRAHA SUN", 0.39},
                    {"GRAHA PLUTO", WesternAspect.CONJUNCTION, "GRAHA PLUTO", 1.69},
                }) {
                    SynastryRow row = rows.stream().filter(one -> named.apply(one.first()).equals(contact[0])
                            && one.aspect() == contact[1] && named.apply(one.second()).equals(contact[2]))
                            .findFirst().orElseThrow(() -> new AssertionError("no contact " + List.of(contact)));
                    check(Math.abs(row.fromExactDeg() - (double) contact[3]) < 0.01, "the recast's " + row);
                }
                check(rows.stream().allMatch(row -> row.fromExactDeg() <= row.orbDeg()), "every contact in orb");
                check(with.apply(Map.of("lagna", false)).synastry().orElseThrow().stream()
                        .allMatch(row -> row.first().graha() != null && row.second().graha() != null),
                        "the lagna left out");
                check(plain.synastryParallels().isEmpty() && plain.synastryAntiscia().isEmpty()
                        && plain.synastryMidpoints().isEmpty() && plain.synastryComposite().isEmpty()
                        && plain.synastryDavison().isEmpty(), "nothing more unless asked");

                // The parallels: Uranus with Uranus closest, a contrary pair within 0.95°.
                List<SynastryParallelRow> parallels = with.apply(Map.of("parallels", Map.of()))
                        .synastryParallels().orElseThrow();
                same("GRAHA URANUS GRAHA URANUS",
                        named.apply(parallels.get(0).first()) + " " + named.apply(parallels.get(0).second()),
                        "the closest parallel");
                check(Math.abs(parallels.get(0).apartDeg() - 0.049) < 0.005, "0.049° apart");
                check(parallels.stream().anyMatch(row -> row.contrary() && row.apartDeg() < 0.95), "a contrary pair");
                check(parallels.stream().allMatch(row -> row.apartDeg() <= row.orbDeg() && row.orbDeg() == 1),
                        "within the 1° orb");

                // The antiscia under Lilly's moieties, closest first.
                List<AntiscionRow> reflected = with.apply(Map.of("antiscia", Map.of())).synastryAntiscia()
                        .orElseThrow();
                same(List.of("SATURN JUPITER false", "SATURN MOON false", "MERCURY MARS false", "MARS SATURN true",
                        "VENUS MARS false", "MARS MERCURY false", "MARS SUN false"),
                        reflected.stream().map(row -> row.first() + " " + row.second() + " " + row.contrary()).toList(),
                        "the recast's seven");
                check(Math.abs(reflected.get(0).apartDeg() - 0.089) < 0.005, "Saturn on Jupiter, 0.089°");

                // The equal distances: her Venus on his Sun and Neptune the closest.
                List<SynastryMidpointRow> equal = with.apply(Map.of("midpoints", Map.of("orbDeg", 1.0)))
                        .synastryMidpoints().orElseThrow();
                same(10, equal.size(), "ten within 1°");
                SynastryMidpointRow closest = equal.get(0);
                same(List.of(Graha.SUN, Graha.NEPTUNE, Graha.VENUS, true, false),
                        List.of(closest.first(), closest.second(), closest.middle(), closest.partnersPair(),
                                closest.far()), "the closest");
                check(Math.abs(closest.fromAxisDeg() - 0.14) < 0.01, "0.14° from the axis");

                Chart both = with.apply(Map.of("composite", true, "davison", true));
                Composite composite = both.synastryComposite().orElseThrow();
                same(10, composite.planets().size(), "ten planets");
                Map<Graha, Double> expected = Map.of(Graha.SUN, 68.8193, Graha.MOON, 259.7289, Graha.MARS, 130.5171,
                        Graha.PLUTO, 44.2555);
                for (CompositePlanet planet : composite.planets()) {
                    Double longitude = expected.get(planet.graha());
                    check(longitude == null || turn.applyAsDouble(planet.longitudeDeg() - longitude) < 0.01,
                            planet.toString());
                }
                check(turn.applyAsDouble(composite.midheavenDeg() - 258.1797) < 0.01, "the midheaven");
                check(turn.applyAsDouble(composite.lagnaDeg() - 334.007) < 0.01, "the lagna");
                check(!composite.lagnaTurned(), "not turned");
                List<Double> cusps = composite.cuspsDeg();
                same(12, cusps.size(), "twelve cusps");
                check(turn.applyAsDouble(cusps.get(2) - 57.8601) < 0.01, "the third cusp");
                same(List.of(composite.lagnaDeg(), composite.midheavenDeg()), List.of(cusps.get(0), cusps.get(9)),
                        "the first and tenth are its lagna and midheaven");

                // The Davison birth founds a chart as a birth does.
                DavisonBirth davison = both.synastryDavison().orElseThrow();
                check(Math.abs(davison.instant() - 2_402_752.026736111) < 1e-8, "the mean instant");
                check(Math.abs(davison.place().longitudeDeg().value() - -0.1622) < 1e-9, "the mean longitude");
                same(0, davison.utcOffsetSeconds(), "the mean clock");
                Chart between = sky.chart().found(davison.instant(), davison.place(), davison.utcOffsetSeconds(),
                        ChartOptions.builder().build());
                double mars = between.grahas().stream().filter(one -> one.graha() == Graha.MARS).findFirst()
                        .orElseThrow().tropicalDeg();
                check(turn.applyAsDouble(mars - 20.1267) < 0.01, "Mars in the Davison chart: " + mars);

                // A batch reads each chart as the chart alone reads it.
                Map<String, Object> two = Map.of("partner", mary, "aspects", List.of("SEXTILE", "OPPOSITION"),
                        "parallels", Map.of(), "antiscia", Map.of(), "midpoints", Map.of(), "composite", true,
                        "davison", true);
                double[] instants = {birth, birth - 3000.25};
                ChartBatch batch = sky.chart().foundMany(instants, george, 0,
                        ChartOptions.builder().outerPlanets(true).synastry(PartnerSynastry.of(two)).build());
                for (int k = 0; k < instants.length; k += 1) {
                    Chart one = found.apply(instants[k], two);
                    Chart many = batch.at(k);
                    same(one.synastry(), many.synastry(), "synastry " + k);
                    same(one.synastryParallels(), many.synastryParallels(), "parallels " + k);
                    same(one.synastryAntiscia(), many.synastryAntiscia(), "antiscia " + k);
                    same(one.synastryMidpoints(), many.synastryMidpoints(), "midpoints " + k);
                    same(one.synastryComposite(), many.synastryComposite(), "composite " + k);
                    same(one.synastryDavison(), many.synastryDavison(), "davison " + k);
                }

                same("synastry.partner", refusedField(() -> found.apply(birth, Map.of("partner",
                        Map.of("instant", 9_000_000.0, "utcOffsetSeconds", 0,
                                "place", Map.of("latitude", 51.5058, "longitude", -0.1878, "altitude", 0.0))))),
                        "a partner out of range");
                same("synastry.parallels.orbDeg", refusedField(() -> with.apply(Map.of("parallels",
                        Map.of("orbDeg", 11.0)))), "a parallel orb too wide");
            }
        });
    }
}
