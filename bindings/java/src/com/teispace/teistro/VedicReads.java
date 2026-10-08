package com.teispace.teistro;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.function.IntToLongFunction;
import java.util.function.IntUnaryOperator;

import com.teispace.teistro.blob.Charts;
import com.teispace.teistro.blob.Day;

/**
 * The Vedic readings of a {@link Chart}: each method reads the batch's
 * columns at the chart's index into the records an application wants, as
 * the Python façade's {@code Chart} properties do. What a batch parses once
 * however many charts read it goes through {@link ChartBatch#cached}.
 */
final class VedicReads {
    /** The convention column's value for a custom sunrise altitude. */
    private static final int CUSTOM_SUNRISE = 0xFF;

    private VedicReads() {
    }

    // ---- shared helpers ----

    /** One chart's entry of a per-batch table, or empty when the table does not reach it. */
    static <T> Optional<T> at(List<T> parsed, int index) {
        return index < parsed.size() ? Optional.of(parsed.get(index)) : Optional.empty();
    }

    /** Where a chart's rows begin in a section ragged by a count column: the counts before it, summed. */
    static int start(IntToLongFunction counts, int index) {
        long sum = 0;
        for (int at = 0; at < index; at += 1) {
            sum += counts.applyAsLong(at);
        }
        return Math.toIntExact(sum);
    }

    /** A count column's sum over {@code length} rows. */
    static long sum(IntToLongFunction counts, int length) {
        long sum = 0;
        for (int at = 0; at < length; at += 1) {
            sum += counts.applyAsLong(at);
        }
        return sum;
    }

    /**
     * The members of a bit set over a small closed enum, in the order given
     * (id order): bit {@code n} is the member with id {@code n}. A member with
     * a negative id, the {@code UNKNOWN} sentinel a catalogue enum carries, is
     * never in a set.
     */
    @SafeVarargs
    static <E extends Member> List<E> members(long bits, E... of) {
        List<E> found = new ArrayList<>();
        for (E member : of) {
            int id = member.id();
            if (id >= 0 && id < 64 && (bits & (1L << id)) != 0) {
                found.add(member);
            }
        }
        return List.copyOf(found);
    }

    /** The grahas with ids 0 to {@code count - 1}: the seven, Sun to Saturn, or the nine, to Ketu. */
    static Graha[] grahaIds(int count) {
        Graha[] grahas = new Graha[count];
        for (int id = 0; id < count; id += 1) {
            grahas[id] = Graha.of(id);
        }
        return grahas;
    }

    /** Twelve cells of an int column from {@code start}, one a sign. */
    private static List<Integer> twelve(IntUnaryOperator column, int start) {
        List<Integer> cells = new ArrayList<>(12);
        for (int at = start; at < start + 12; at += 1) {
            cells.add(column.applyAsInt(at));
        }
        return List.copyOf(cells);
    }

    private static Charts decoded(Chart chart) {
        return chart.batch().decoded();
    }

    // ---- the day and the timing ----

    /**
     * The day the chart's moment belongs to, which may be the civil date
     * before the instant's: its date, weekday and sunrises. Ports {@code Chart.day}.
     */
    static LocalDay day(Chart chart) {
        return localDay(decoded(chart).day(), chart.index());
    }

    /**
     * One row of a {@code day} section, a chart's or an almanac's, which share
     * it, read into the record both layers hand back.
     */
    static LocalDay localDay(Day section, int i) {
        boolean custom = section.conventionKind(i) == CUSTOM_SUNRISE;
        boolean polar = DayState.of(section.stateKind(i)) == DayState.POLAR;
        CalendarDate date = new CalendarDate(
                Calendar.of(section.calendar(i)),
                section.era(i) == Values.NO_MEMBER ? null : Era.of(section.era(i)),
                section.year(i),
                section.eraYear(i),
                section.month(i),
                section.dayOfMonth(i),
                Resolution.of(section.resolution(i)),
                section.computedMonth(i),
                section.computedDay(i));
        return new LocalDay(
                date,
                Vara.of(section.vara(i)),
                section.sunrise(i),
                section.sunset(i),
                section.nextSunrise(i),
                polar
                        ? new PolarDay(PolarKind.of(section.statePolarKind(i)),
                                PolarDayPolicy.of(section.statePolarPolicy(i)))
                        : null,
                custom ? null : Sunrise.of(section.conventionKind(i)),
                custom ? Double.valueOf(section.conventionValue(i)) : null,
                // No air has a pressure of zero, so a zero says the convention named none.
                section.airPressureHpa(i) > 0 ? new Air(section.airPressureHpa(i), section.airTemperatureC(i)) : null);
    }

    /** Where in its day the moment falls, in the reckonings the settings named. Ports {@code Chart.timing}. */
    static ChartTiming timing(Chart chart) {
        Charts.Timing t = decoded(chart).timing();
        int i = chart.index();
        return new ChartTiming(t.ghati(i), t.pala(i), t.vipala(i), GhatiReckoning.of(t.ghatiReckoning(i)),
                t.horaNumber(i), Graha.of(t.horaLord(i)), t.horaStart(i), t.horaEnd(i));
    }

    // ---- states, bhavas, points, aspects ----

    /**
     * What each graha <b>is</b>, as opposed to where it is, or an empty list
     * unless {@code state} asked for it. The motion is not here:
     * {@code grahas().get(j).retrograde()} already says it. Ports {@code Chart.states}.
     */
    static List<GrahaState> states(Chart chart) {
        Charts decoded = decoded(chart);
        Charts.States c = decoded.states();
        if (c.length() == 0) {
            return List.of();
        }
        int count = Math.toIntExact(decoded.grahaCount());
        int base = chart.index() * count;
        AvasthaLajjitadi[] lajjitadi = AvasthaLajjitadi.values();
        List<GrahaState> states = new ArrayList<>(count);
        for (int i = base; i < base + count; i += 1) {
            states.add(new GrahaState(
                    Graha.of(c.graha(i)),
                    Rashi.of(c.sign(i)),
                    c.house(i),
                    Dignity.of(c.dignity(i)),
                    new Friendship(
                            Relationship.of(c.natural(i)),
                            Relationship.of(c.temporary(i)),
                            Relationship.of(c.compound(i)),
                            c.hasDispositor(i) != 0 ? Graha.of(c.dispositor(i)) : null),
                    new Combustion(
                            Burning.of(c.burning(i)),
                            c.hasFromSun(i) != 0 ? Double.valueOf(c.fromSunDeg(i)) : null,
                            c.hasOrbs(i) != 0 ? Double.valueOf(c.orbDeg(i)) : null,
                            c.hasDeepOrb(i) != 0 ? Double.valueOf(c.deepOrbDeg(i)) : null),
                    AvasthaBaladi.of(c.age(i)),
                    AvasthaJagradadi.of(c.wakefulness(i)),
                    c.hasDeeptadi(i) != 0 ? AvasthaDeeptadi.of(c.deeptadi(i)) : null,
                    new Lajjitadi(
                            members(c.lajjitadiHolding(i), lajjitadi),
                            members(c.lajjitadiRuledOut(i), lajjitadi),
                            members(c.lajjitadiUndecided(i), lajjitadi)),
                    c.hasWar(i) != 0 ? new War(Graha.of(c.warOpponent(i)), c.warWon(i) != 0, c.warApartDeg(i)) : null,
                    c.hasSayanadi(i) != 0
                            ? new Sayanadi(AvasthaSayanadi.of(c.sayanadi(i)), List.of(
                                    AvasthaCheshta.of(c.cheshta1(i)),
                                    AvasthaCheshta.of(c.cheshta2(i)),
                                    AvasthaCheshta.of(c.cheshta3(i)),
                                    AvasthaCheshta.of(c.cheshta4(i)),
                                    AvasthaCheshta.of(c.cheshta5(i))))
                            : null,
                    new EdgeDistance(c.signDeg(i), c.nakshatraDeg(i), c.padaDeg(i))));
        }
        return List.copyOf(states);
    }

    /**
     * The twelve bhavas as the houses service reads them, or an empty list
     * unless {@code houses} asked for them. Ports {@code Chart.bhavas}.
     */
    static List<ServiceBhava> bhavas(Chart chart) {
        Charts.Bhavas columns = decoded(chart).bhavas();
        if (columns.length() == 0) {
            return List.of();
        }
        int base = chart.index() * 12;
        List<ServiceBhava> bhavas = new ArrayList<>(12);
        for (int j = 0; j < 12; j += 1) {
            bhavas.add(new ServiceBhava(j + 1, Rashi.of(columns.sign(base + j)), Graha.of(columns.lord(base + j)),
                    Quadrant.of(columns.quadrant(base + j))));
        }
        return List.copyOf(bhavas);
    }

    /**
     * The derived points, the upagrahas and the special lagnas, or an empty
     * list unless {@code points} asked for them. The section is ragged:
     * Gulika and Mandi are Saturn's eighth of the day's arc, so a chart with
     * no arc to divide cannot have them. Ports {@code Chart.points}.
     */
    static List<DerivedPoint> points(Chart chart) {
        Charts decoded = decoded(chart);
        Charts.Cast cast = decoded.cast();
        int start = start(cast::pointCount, chart.index());
        int count = Math.toIntExact(cast.pointCount(chart.index()));
        Charts.Points c = decoded.points();
        List<DerivedPoint> points = new ArrayList<>(count);
        for (int i = start; i < start + count; i += 1) {
            points.add(new DerivedPoint(Point.of(c.point(i)), c.longitudeDeg(i), Rashi.of(c.sign(i)),
                    new EdgeDistance(c.signDeg(i), c.nakshatraDeg(i), c.padaDeg(i))));
        }
        return List.copyOf(points);
    }

    /**
     * The drishti this chart casts, strongest first among those a body casts;
     * empty unless {@code aspects} asked for them. The section is ragged by
     * {@code cast.aspect_count}. Ports {@code Chart.aspects}.
     */
    static List<Drishti> aspects(Chart chart) {
        Charts decoded = decoded(chart);
        Charts.Cast cast = decoded.cast();
        int start = start(cast::aspectCount, chart.index());
        int count = Math.toIntExact(cast.aspectCount(chart.index()));
        Charts.Aspects c = decoded.aspects();
        List<Drishti> aspects = new ArrayList<>(count);
        for (int i = start; i < start + count; i += 1) {
            aspects.add(new Drishti(
                    Graha.of(c.from(i)),
                    Graha.of(c.to(i)),
                    c.houses(i),
                    Strength.of(c.strength(i)),
                    new EdgeDistance(c.fromSignDeg(i), c.fromNakshatraDeg(i), c.fromPadaDeg(i)),
                    new EdgeDistance(c.toSignDeg(i), c.toNakshatraDeg(i), c.toPadaDeg(i))));
        }
        return List.copyOf(aspects);
    }

    // ---- ashtakavarga and the balas ----

    /** The Ashtakavarga, when {@code ashtakavarga} asked for it. Ports {@code Chart.ashtakavarga}. */
    static Optional<Ashtakavarga> ashtakavarga(Chart chart) {
        return at(chart.batch().cached("ashtakavargas", () -> ashtakavargas(decoded(chart))), chart.index());
    }

    /** Every chart's Ashtakavarga, decoded once; empty when none was asked for. */
    private static List<Ashtakavarga> ashtakavargas(Charts decoded) {
        Charts.Ashtakavarga rows = decoded.ashtakavarga();
        Charts.AshtakavargaBindus bins = decoded.ashtakavargaBindus();
        Charts.Sarvashtakavarga sums = decoded.sarvashtakavarga();
        List<Ashtakavarga> out = new ArrayList<>();
        for (int chart = 0; chart < rows.length() / 7; chart += 1) {
            List<GrahaAshtakavarga> grahas = new ArrayList<>(7);
            for (int g = 0; g < 7; g += 1) {
                int row = chart * 7 + g;
                boolean each = Shodhana.of(rows.shodhana(row)) == Shodhana.EACH_GRAHA;
                grahas.add(new GrahaAshtakavarga(
                        Graha.of(rows.graha(row)),
                        twelve(bins::bindus, row * 12),
                        each ? twelve(bins::reduced, row * 12) : null,
                        rows.rashiPinda(row),
                        rows.grahaPinda(row),
                        rows.yogaPinda(row)));
            }
            out.add(new Ashtakavarga(
                    Shodhana.of(rows.shodhana(chart * 7)),
                    Ekadhipatya.of(rows.ekadhipatya(chart * 7)),
                    grahas,
                    twelve(sums::sarva, chart * 12),
                    twelve(sums::trikona, chart * 12),
                    twelve(sums::reduced, chart * 12)));
        }
        return List.copyOf(out);
    }

    /** The Bhava bala, when {@code bhava_bala} asked for it. Ports {@code Chart.bhava_bala}. */
    static Optional<BhavaBala> bhavaBala(Chart chart) {
        return at(chart.batch().cached("bhavaBalas", () -> bhavaBalas(decoded(chart))), chart.index());
    }

    private static List<BhavaBala> bhavaBalas(Charts decoded) {
        Charts.BhavaBala c = decoded.bhavaBala();
        List<BhavaBala> out = new ArrayList<>();
        for (int chart = 0; chart < c.length() / 12; chart += 1) {
            List<BhavaStrength> bhavas = new ArrayList<>(12);
            for (int h = 0; h < 12; h += 1) {
                int row = chart * 12 + h;
                bhavas.add(new BhavaStrength(h + 1, Graha.of(c.lord(row)), c.adhipati(row), c.dig(row), c.drishti(row),
                        c.special(row), c.virupas(row)));
            }
            out.add(new BhavaBala(bhavas));
        }
        return List.copyOf(out);
    }

    /** The Shadbala, when {@code shadbala} asked for it. Ports {@code Chart.shadbala}. */
    static Optional<Shadbala> shadbala(Chart chart) {
        return at(chart.batch().cached("shadbalas", () -> shadbalas(decoded(chart))), chart.index());
    }

    private static List<Shadbala> shadbalas(Charts decoded) {
        Charts.Shadbala c = decoded.shadbala();
        List<Shadbala> out = new ArrayList<>();
        for (int chart = 0; chart < c.length() / 7; chart += 1) {
            List<GrahaShadbala> grahas = new ArrayList<>(7);
            for (int row = chart * 7; row < chart * 7 + 7; row += 1) {
                grahas.add(new GrahaShadbala(
                        Graha.of(c.graha(row)),
                        new SthanaBala(c.uchcha(row), c.saptavargaja(row), c.ojayugma(row), c.kendradi(row),
                                c.drekkana(row)),
                        c.dig(row),
                        new KaalaBala(c.nathonnatha(row), c.paksha(row), c.tribhaga(row), c.abda(row), c.masa(row),
                                c.vara(row), c.hora(row), c.ayana(row), c.yuddha(row)),
                        c.cheshta(row),
                        c.naisargika(row),
                        c.drik(row),
                        c.virupas(row),
                        c.rupas(row),
                        c.requiredRupas(row),
                        c.strong(row) == 1,
                        c.ishta(row),
                        c.kashta(row),
                        c.subhaRashmi(row),
                        c.ashubhaRashmi(row)));
            }
            out.add(new Shadbala(grahas));
        }
        return List.copyOf(out);
    }

    /** The dasha phala, when {@code dasha_phala} asked for it. Ports {@code Chart.dasha_phala}. */
    static Optional<DashaPhalaReading> dashaPhala(Chart chart) {
        return at(chart.batch().cached("dashaPhalas", () -> dashaPhalas(decoded(chart))), chart.index());
    }

    private static List<DashaPhalaReading> dashaPhalas(Charts decoded) {
        Charts.DashaPhala c = decoded.dashaPhala();
        List<DashaPhalaReading> out = new ArrayList<>();
        for (int chart = 0; chart < c.length() / 9; chart += 1) {
            List<GrahaDashaPhala> grahas = new ArrayList<>(9);
            for (int row = chart * 9; row < chart * 9 + 9; row += 1) {
                grahas.add(new GrahaDashaPhala(
                        Graha.of(c.graha(row)),
                        List.of(c.subhankaD1(row), c.subhankaD2(row), c.subhankaD3(row), c.subhankaD7(row),
                                c.subhankaD9(row), c.subhankaD12(row), c.subhankaD30(row)),
                        c.subhanka(row),
                        c.asubhanka(row),
                        Nature.of(c.nature(row)),
                        DashaPhase.of(c.phase(row)),
                        c.favourable(row) == 1,
                        c.unfavourable(row) == 1));
            }
            out.add(new DashaPhalaReading(grahas));
        }
        return List.copyOf(out);
    }

    // ---- hits, sade sati, gochar ----

    /**
     * The transit hit list, sorted by instant, then graha, then kind; empty
     * unless {@code hits} asked for it. The section is ragged by
     * {@code cast.hit_count}. Ports {@code Chart.hits}.
     *
     * @throws IllegalStateException when the section's rows are not every chart's list
     */
    static List<Hit> hits(Chart chart) {
        Charts decoded = decoded(chart);
        Charts.Cast cast = decoded.cast();
        Charts.Hits columns = decoded.hits();
        long total = sum(cast::hitCount, cast.length());
        if (total != columns.length()) {
            throw new IllegalStateException("hits has " + columns.length() + " rows and cast.hit_count sums to "
                    + total + "; it is every chart's list, concatenated");
        }
        int start = start(cast::hitCount, chart.index());
        int count = Math.toIntExact(cast.hitCount(chart.index()));
        List<Hit> hits = new ArrayList<>(count);
        for (int row = start; row < start + count; row += 1) {
            hits.add(hitAt(columns, row));
        }
        return List.copyOf(hits);
    }

    /** One row of the {@code hits} section as the Rust {@code Hit} spells it. */
    private static Hit hitAt(Charts.Hits columns, int row) {
        HitKind kind = HitKind.of(columns.kind(row));
        Motion motion = Motion.of(columns.motion(row));
        HitEvent event = switch (kind) {
            case SIGN_INGRESS -> new SignIngress(Rashi.of(columns.into(row)), motion);
            case NAKSHATRA_INGRESS -> new NakshatraIngress(Nakshatra.of(columns.into(row)), motion);
            case STATION -> new Station(motion);
            default -> new AspectHit(pointAt(columns.toLagna(row), columns.toGraha(row)), columns.angle(row),
                    AspectPhase.of(columns.phase(row)), motion);
        };
        return new Hit(columns.instant(row), Graha.of(columns.graha(row)), event);
    }

    /** A natal point from a {@code to_lagna} and a {@code to_graha} column's cells. */
    static NatalPoint pointAt(int toLagna, int toGraha) {
        return toLagna != 0 ? new NatalPoint("LAGNA", null) : new NatalPoint("GRAHA", Graha.of(toGraha));
    }

    /**
     * Sade Sati and Saturn's smaller spells, every period reaching into the
     * window <b>whole</b>; empty unless {@code sade_sati} asked for it.
     * Ports {@code Chart.sade_sati}.
     */
    static Optional<SadeSatiReport> sadeSati(Chart chart) {
        return at(chart.batch().cached("sadeSatis", () -> sadeSatis(decoded(chart))), chart.index());
    }

    /** A spell being gathered: its house and its visits so far. */
    private record Run(int house, List<SadeSatiVisit> visits) {
        SadeSatiSpell spell() {
            return new SadeSatiSpell(house, visits);
        }
    }

    private static Double bound(double jd) {
        return Double.isNaN(jd) ? null : jd;
    }

    /**
     * Every chart's Sade Sati report, decoded once; empty when none was asked
     * for. {@code sade_sati} holds a row a chart and {@code sade_sati_visits}
     * each chart's visits, ragged by {@code cast.sade_sati_visit_count} and
     * numbered by {@code period}: its Sade Satis first (houses 12, 1 and 2),
     * then its smaller spells.
     */
    private static List<SadeSatiReport> sadeSatis(Charts decoded) {
        Charts.SadeSati c = decoded.sadeSati();
        Charts.SadeSatiVisits v = decoded.sadeSatiVisits();
        Charts.Cast cast = decoded.cast();
        if (c.length() == 0) {
            return List.of();
        }
        long total = sum(cast::sadeSatiVisitCount, cast.length());
        if (c.length() != cast.length() || total != v.length()) {
            throw JsonRead.internal("sade_sati has " + c.length() + " rows and sade_sati_visits " + v.length()
                    + " over " + cast.length() + " charts whose counts sum to " + total
                    + "; it is a row a chart and every chart's visits");
        }
        List<SadeSatiReport> reports = new ArrayList<>(c.length());
        int start = 0;
        for (int chart = 0; chart < c.length(); chart += 1) {
            int count = Math.toIntExact(cast.sadeSatiVisitCount(chart));
            // A period's rows are adjacent and share `period`; a spell's are
            // the run of one house inside it.
            List<List<Run>> periods = new ArrayList<>();
            for (int row = start; row < start + count; row += 1) {
                if (v.period(row) == periods.size()) {
                    periods.add(new ArrayList<>());
                }
                List<Run> spells = periods.get(v.period(row));
                if (spells.isEmpty() || spells.get(spells.size() - 1).house() != v.house(row)) {
                    spells.add(new Run(v.house(row), new ArrayList<>()));
                }
                spells.get(spells.size() - 1).visits().add(new SadeSatiVisit(bound(v.from(row)), bound(v.to(row))));
            }
            List<SadeSati> sadeSati = new ArrayList<>();
            List<SadeSatiSpell> smaller = new ArrayList<>();
            for (List<Run> spells : periods) {
                // The Sade Sati's houses are 12, 1 and 2; a smaller spell is 3 to 11.
                int first = spells.get(0).house();
                if (first == 12 || first <= 2) {
                    sadeSati.add(new SadeSati(spells.stream().map(Run::spell).toList()));
                } else {
                    smaller.add(spells.get(0).spell());
                }
            }
            reports.add(new SadeSatiReport(
                    new GocharReference(GocharFrom.of(c.countedFrom(chart)), Rashi.of(c.reference(chart))),
                    Reckoning.of(c.reckoning(chart)),
                    sadeSati,
                    smaller));
            start += count;
        }
        return List.copyOf(reports);
    }

    /**
     * The transits read against this chart, one reading an instant in the
     * order {@code gochar.instants} asked; empty unless asked for. Ports
     * {@code Chart.gochar}.
     */
    static List<GocharReading> gochar(Chart chart) {
        List<List<GocharReading>> parsed = chart.batch().cached("gochars", () -> gochars(decoded(chart)));
        return chart.index() < parsed.size() ? parsed.get(chart.index()) : List.of();
    }

    /**
     * Every chart's transits, decoded once. Fixed rather than ragged: the
     * request settles how many instants every chart gets, so each chart holds
     * the section's rows over the chart count.
     */
    private static List<List<GocharReading>> gochars(Charts decoded) {
        Charts.Gochar c = decoded.gochar();
        Charts.GocharGrahas g = decoded.gocharGrahas();
        Charts.GocharAshtakavarga a = decoded.gocharAshtakavarga();
        int charts = Math.toIntExact(decoded.chartCount());
        int perChart = charts == 0 ? 0 : c.length() / charts;
        int left = charts == 0 ? 0 : c.length() % charts;
        if (left != 0 || g.length() != (long) c.length() * 9) {
            throw JsonRead.internal("gochar has " + c.length() + " rows and " + g.length() + " grahas over " + charts
                    + " charts; it is every chart at every instant, nine grahas each");
        }
        boolean judged = a.length() > 0;
        if (judged && a.length() != (long) c.length() * 7) {
            throw JsonRead.internal("gochar_ashtakavarga has " + a.length() + " rows under " + c.length()
                    + " transits; it is seven under every one or none");
        }
        Graha[] nine = grahaIds(9);
        List<List<GocharReading>> out = new ArrayList<>(charts);
        for (int chart = 0; chart < charts; chart += 1) {
            List<GocharReading> readings = new ArrayList<>(perChart);
            for (int k = 0; k < perChart; k += 1) {
                int row = chart * perChart + k;
                List<GrahaGochar> grahas = new ArrayList<>(9);
                for (int n = 0; n < 9; n += 1) {
                    int at = row * 9 + n;
                    int vedha = g.vedhaHouse(at);
                    grahas.add(new GrahaGochar(
                            Graha.of(g.graha(at)),
                            new Transit(Rashi.of(g.sign(at)), g.degrees(at)),
                            g.house(at),
                            g.goodHouse(at) == 1,
                            vedha != 0 ? Integer.valueOf(vedha) : null,
                            members(g.obstructedBy(at), nine),
                            GocharVerdict.of(g.verdict(at)),
                            Fruition.of(g.fruition(at)),
                            g.fruitfulNow(at) == 1));
                }
                List<AshtakavargaTransit> byBindus = null;
                if (judged) {
                    byBindus = new ArrayList<>(7);
                    for (int at = row * 7; at < row * 7 + 7; at += 1) {
                        byBindus.add(new AshtakavargaTransit(
                                Graha.of(a.graha(at)),
                                a.bindus(at),
                                a.good(at) == 1,
                                new Kakshya(a.kakshya(at), KakshyaLord.of(a.kakshyaLord(at))),
                                a.kakshyaBindu(at) == 1,
                                a.sarva(at),
                                SarvaStanding.of(a.sarvaStanding(at))));
                    }
                }
                readings.add(new GocharReading(
                        c.instant(row),
                        new GocharReference(GocharFrom.of(c.countedFrom(row)), Rashi.of(c.reference(row))),
                        new GocharRules(NodeVedha.of(c.nodeVedha(row)), NodeObstruction.of(c.nodeObstruction(row)),
                                AshtakavargaGoodFrom.of(c.ashtakavargaGoodFrom(row))),
                        grahas,
                        byBindus));
            }
            out.add(List.copyOf(readings));
        }
        return List.copyOf(out);
    }

    // ---- jaimini, avakahada, vaiseshikamsa, vimshopaka ----

    /** Jaimini's significators, when {@code jaimini} asked for them. Ports {@code Chart.jaimini}. */
    static Optional<JaiminiReading> jaimini(Chart chart) {
        return at(chart.batch().cached("jaiminis", () -> jaiminis(decoded(chart))), chart.index());
    }

    private static List<JaiminiReading> jaiminis(Charts decoded) {
        Charts.Jaimini c = decoded.jaimini();
        Charts.JaiminiGrahas h = decoded.jaiminiGrahas();
        Graha[] nine = grahaIds(9);
        List<JaiminiReading> out = new ArrayList<>(c.length());
        for (int chart = 0; chart < c.length(); chart += 1) {
            BrahmaOutcome outcome = BrahmaOutcome.of(c.brahmaOutcome(chart));
            boolean found = outcome == BrahmaOutcome.FOUND;
            List<Integer> inRasi = new ArrayList<>(9);
            List<Integer> inNavamsha = new ArrayList<>(9);
            List<Rashi> arudhas = new ArrayList<>(9);
            for (int row = chart * 9; row < chart * 9 + 9; row += 1) {
                inRasi.add(h.inRasi(row));
                inNavamsha.add(h.inNavamsha(row));
                arudhas.add(h.arudhaPresent(row) == 1 ? Rashi.of(h.arudha(row)) : null);
            }
            out.add(new JaiminiReading(
                    new Karakamsha(Graha.of(c.atmakaraka(chart)), Rashi.of(c.karakamsha(chart)), inRasi, inNavamsha),
                    new Brahma(
                            BrahmaRule.of(c.brahmaRule(chart)),
                            Rashi.of(c.countedFrom(chart)),
                            members(c.qualified(chart), nine),
                            found ? Graha.of(c.brahma(chart)) : null,
                            c.passedFromPresent(chart) == 1 ? Graha.of(c.passedFrom(chart)) : null,
                            found ? null : outcome),
                    Collections.unmodifiableList(arudhas)));
        }
        return List.copyOf(out);
    }

    /**
     * The Moon's avakahada, when {@code avakahada} asked for it; a tropical
     * chart refuses it, named {@code avakahada}. Ports {@code Chart.avakahada}.
     */
    static Optional<Avakahada> avakahada(Chart chart) {
        return at(chart.batch().cached("avakahadas", () -> avakahadas(decoded(chart))), chart.index());
    }

    private static List<Avakahada> avakahadas(Charts decoded) {
        Charts.Avakahada c = decoded.avakahada();
        String text = decoded.avakahadaSyllables();
        List<?> syllables = text.isEmpty() ? List.of() : JsonRead.list(Json.read(text));
        List<Avakahada> out = new ArrayList<>(c.length());
        for (int chart = 0; chart < c.length(); chart += 1) {
            List<?> syllable = JsonRead.list(syllables.get(chart));
            out.add(new Avakahada(
                    Nakshatra.of(c.nakshatra(chart)),
                    c.pada(chart),
                    Rashi.of(c.rashi(chart)),
                    Graha.of(c.nakshatraLord(chart)),
                    Graha.of(c.rashiLord(chart)),
                    Varna.of(c.varna(chart)),
                    Yoni.of(c.yoni(chart)),
                    Gana.of(c.gana(chart)),
                    Nadi.of(c.nadi(chart)),
                    new BirthSyllable(c.cell(chart), JsonRead.text(syllable.get(0)), JsonRead.text(syllable.get(1)),
                            NameVarga.of(c.varga(chart)))));
        }
        return List.copyOf(out);
    }

    /** The Vaiseshikamsa, when {@code vaiseshikamsa} asked for it. Ports {@code Chart.vaiseshikamsa}. */
    static Optional<VaiseshikamsaReading> vaiseshikamsa(Chart chart) {
        return at(chart.batch().cached("vaiseshikamsas", () -> vaiseshikamsas(decoded(chart))), chart.index());
    }

    private static VaiseshikamsaStanding standing(int count, int name) {
        return new VaiseshikamsaStanding(count, count >= 2 ? Vaiseshikamsa.of(name) : null);
    }

    private static List<VaiseshikamsaReading> vaiseshikamsas(Charts decoded) {
        Charts.Vaiseshikamsa c = decoded.vaiseshikamsa();
        List<VaiseshikamsaReading> out = new ArrayList<>();
        for (int chart = 0; chart < c.length() / 7; chart += 1) {
            List<GrahaVaiseshikamsa> grahas = new ArrayList<>(7);
            for (int row = chart * 7; row < chart * 7 + 7; row += 1) {
                grahas.add(new GrahaVaiseshikamsa(
                        Graha.of(c.graha(row)),
                        standing(c.shadvargaGood(row), c.shadvargaName(row)),
                        standing(c.saptavargaGood(row), c.saptavargaName(row)),
                        standing(c.dashavargaGood(row), c.dashavargaName(row)),
                        standing(c.shodashavargaGood(row), c.shodashavargaName(row)),
                        c.impaired(row) == 1));
            }
            out.add(new VaiseshikamsaReading(grahas));
        }
        return List.copyOf(out);
    }

    /** The Vimshopaka, when {@code vimshopaka} asked for it. Ports {@code Chart.vimshopaka}. */
    static Optional<Vimshopaka> vimshopaka(Chart chart) {
        return at(chart.batch().cached("vimshopakas", () -> vimshopakas(decoded(chart))), chart.index());
    }

    private static List<Vimshopaka> vimshopakas(Charts decoded) {
        Charts.Vimshopaka rows = decoded.vimshopaka();
        List<Vimshopaka> out = new ArrayList<>();
        for (int chart = 0; chart < rows.length() / 7; chart += 1) {
            List<GrahaVimshopaka> grahas = new ArrayList<>(7);
            for (int row = chart * 7; row < chart * 7 + 7; row += 1) {
                grahas.add(new GrahaVimshopaka(Graha.of(rows.graha(row)), rows.shadvarga(row), rows.saptavarga(row),
                        rows.dashavarga(row), rows.shodashavarga(row)));
            }
            out.add(new Vimshopaka(VimshopakaScoring.of(rows.scoring(chart * 7)), grahas));
        }
        return List.copyOf(out);
    }

    // ---- rules, plans, drawings ----

    /** Every chart's object of a JSON section that holds a list of them, parsed once; empty for no text. */
    private static List<Map<String, Object>> objects(String text) {
        if (text.isEmpty()) {
            return List.of();
        }
        return JsonRead.list(Json.read(text)).stream().map(JsonRead::object).toList();
    }

    /**
     * What this chart answers by rule, as the SDK writes it: {@code present},
     * each {@code {rule, result}} with the rule by key, and {@code houses} and
     * {@code longevity} when asked; empty unless the request named rules.
     * Ports {@code Chart.rules}.
     */
    static Optional<Map<String, Object>> rules(Chart chart) {
        return at(chart.batch().cached("rules", () -> objects(decoded(chart).rules())), chart.index());
    }

    /**
     * What this chart has to say, as the composers wrote it: a key per
     * composer the request asked for, each a list of {@code {key, params}}
     * holding no words at all. An item's {@code params} are the mapping
     * {@code intl.render} takes. Empty unless the request named a composer.
     * Ports {@code Chart.plans}.
     */
    static Optional<Map<String, Object>> plans(Chart chart) {
        return at(chart.batch().cached("plans", () -> objects(decoded(chart).plans())), chart.index());
    }

    /**
     * The charts drawn in the layouts asked for, in the order asked; empty
     * unless {@code drawings} named some. Ports {@code Chart.drawings}.
     */
    static List<Drawing> drawings(Chart chart) {
        List<List<Drawing>> parsed = chart.batch().cached("drawings", () -> drawingsOf(decoded(chart)));
        return chart.index() < parsed.size() ? parsed.get(chart.index()) : List.of();
    }

    private static List<List<Drawing>> drawingsOf(Charts decoded) {
        String text = decoded.drawings();
        if (text.isEmpty()) {
            return List.of();
        }
        List<?> written = decoded.svgs().isEmpty() ? List.of() : JsonRead.list(Json.read(decoded.svgs()));
        List<?> charts = JsonRead.list(Json.read(text));
        List<List<Drawing>> out = new ArrayList<>(charts.size());
        for (int chart = 0; chart < charts.size(); chart += 1) {
            List<?> drawings = JsonRead.list(charts.get(chart));
            List<Drawing> drawn = new ArrayList<>(drawings.size());
            for (int index = 0; index < drawings.size(); index += 1) {
                String svg = chart < written.size() ? JsonRead.textOrNull(JsonRead.list(written.get(chart)).get(index))
                        : null;
                drawn.add(drawing(drawings.get(index), svg));
            }
            out.add(List.copyOf(drawn));
        }
        return List.copyOf(out);
    }

    private static UnitPoint unitPoint(Object raw) {
        return new UnitPoint(JsonRead.decimal(JsonRead.at(raw, "x")), JsonRead.decimal(JsonRead.at(raw, "y")));
    }

    private static Segment segment(Object raw) {
        String kind = JsonRead.text(JsonRead.at(raw, "kind"));
        return switch (kind) {
            case "LINE" -> new LineSegment(unitPoint(JsonRead.at(raw, "to")));
            case "QUAD" -> new QuadSegment(unitPoint(JsonRead.at(raw, "control")), unitPoint(JsonRead.at(raw, "to")));
            case "ARC" -> new ArcSegment(unitPoint(JsonRead.at(raw, "centre")),
                    JsonRead.bool(JsonRead.at(raw, "clockwise")), unitPoint(JsonRead.at(raw, "to")));
            // A step the SDK does not write is refused, never read as an arc.
            default -> throw new TeistroException(Status.INTERNAL, "an outline step of kind '" + kind + "'", "",
                    "segments", "", "");
        };
    }

    private static Outline outline(Object raw) {
        return new Outline(unitPoint(JsonRead.at(raw, "start")),
                JsonRead.list(JsonRead.at(raw, "segments")).stream().map(VedicReads::segment).toList());
    }

    /** A drawn layout: the shipped member, or a registered one's full key. */
    private static Object layoutOf(String key) {
        Optional<ChartLayout> shipped = ChartLayout.byKey(key);
        return shipped.isPresent() ? shipped.get() : "chart_layout." + key;
    }

    private static Drawing drawing(Object raw, String svg) {
        Object placed = JsonRead.at(raw, "placed");
        List<DrawnCell> cells = new ArrayList<>();
        for (Object cell : JsonRead.list(JsonRead.at(placed, "cells"))) {
            cells.add(new DrawnCell(
                    outline(JsonRead.at(cell, "outline")),
                    JsonRead.member(Rashi::byKey, "Rashi", JsonRead.at(cell, "sign")),
                    JsonRead.integer(JsonRead.at(cell, "house")),
                    JsonRead.bool(JsonRead.at(cell, "lagna")),
                    JsonRead.integer(JsonRead.at(cell, "ring")),
                    unitPoint(JsonRead.at(cell, "label")),
                    unitPoint(JsonRead.at(cell, "anchor")),
                    JsonRead.texts(JsonRead.at(cell, "bodies"))));
        }
        List<DrawnMark> marks = new ArrayList<>();
        for (Object mark : JsonRead.list(JsonRead.at(placed, "marks"))) {
            marks.add(new DrawnMark(
                    JsonRead.text(JsonRead.at(mark, "body")),
                    JsonRead.integer(JsonRead.at(mark, "ring")),
                    unitPoint(JsonRead.at(mark, "at")),
                    JsonRead.decimal(JsonRead.at(mark, "longitude_deg"))));
        }
        return new Drawing(
                layoutOf(JsonRead.text(JsonRead.at(placed, "layout"))),
                JsonRead.member(Varga::byKey, "Varga", JsonRead.at(raw, "varga")),
                cells,
                JsonRead.list(JsonRead.at(placed, "frame")).stream().map(VedicReads::outline).toList(),
                marks,
                svg);
    }

    // ---- vargas, grahas, houses ----

    /**
     * The divisional charts asked for, in the order they were asked; empty
     * unless {@code vargas} named some. Ports {@code Chart.vargas}.
     */
    static List<VargaChart> vargas(Chart chart) {
        Charts decoded = decoded(chart);
        int count = Math.toIntExact(decoded.vargaCount());
        int grahaCount = Math.toIntExact(decoded.grahaCount());
        Charts.Vargas charts = decoded.vargas();
        Charts.VargaGrahas placed = decoded.vargaGrahas();
        Charts.Grahas grahas = decoded.grahas();
        List<VargaChart> out = new ArrayList<>(count);
        for (int at = 0; at < count; at += 1) {
            int row = chart.index() * count + at;
            int base = row * grahaCount;
            List<PlacedInVarga> placings = new ArrayList<>(grahaCount);
            for (int j = 0; j < grahaCount; j += 1) {
                placings.add(new PlacedInVarga(
                        Graha.of(grahas.graha(chart.index() * grahaCount + j)),
                        new VargaPlacement(Rashi.of(placed.rashi(base + j)), placed.part(base + j),
                                Rashi.of(placed.sign(base + j)))));
            }
            out.add(new VargaChart(
                    Varga.of(charts.varga(row)),
                    new VargaPlacement(Rashi.of(charts.lagnaRashi(row)), charts.lagnaPart(row),
                            Rashi.of(charts.lagnaSign(row))),
                    placings));
        }
        return List.copyOf(out);
    }

    /** The grahas, in the catalogue's order, one record each. Ports {@code Chart.grahas}. */
    static List<PlacedGraha> grahas(Chart chart) {
        Charts decoded = decoded(chart);
        Charts.Grahas c = decoded.grahas();
        int count = Math.toIntExact(decoded.grahaCount());
        int base = chart.index() * count;
        List<PlacedGraha> out = new ArrayList<>(count);
        for (int i = base; i < base + count; i += 1) {
            out.add(new PlacedGraha(
                    Graha.of(c.graha(i)), c.longitudeDeg(i), c.tropicalDeg(i), c.latitudeDeg(i), c.distanceAu(i),
                    c.speedDegPerDay(i),
                    new Placement(c.houseBhava(i), HouseSystem.of(c.houseMethod(i)), c.houseThrough(i),
                            c.houseFromMadhyaDeg(i)),
                    new Placement(c.placementBhava(i), HouseSystem.of(c.placementMethod(i)), c.placementThrough(i),
                            c.placementFromMadhyaDeg(i))));
        }
        return List.copyOf(out);
    }

    /**
     * Uranus, Neptune and Pluto, placed as the grahas are, or an empty list
     * unless {@code outer_planets} asked for them. The section holds the same
     * number a chart, so the batch's rows divided by its charts is this
     * chart's count. Ports {@code Chart.outer}.
     *
     * @throws IllegalStateException when the rows do not divide among the charts
     */
    static List<PlacedGraha> outer(Chart chart) {
        Charts decoded = decoded(chart);
        int charts = decoded.cast().length();
        Charts.Outer c = decoded.outer();
        int rows = c.length();
        int count = charts == 0 ? 0 : rows / charts;
        int odd = charts == 0 ? 0 : rows % charts;
        if (odd != 0) {
            throw new IllegalStateException("outer has " + rows + " rows for " + charts + " charts");
        }
        int base = chart.index() * count;
        List<PlacedGraha> out = new ArrayList<>(count);
        for (int i = base; i < base + count; i += 1) {
            out.add(new PlacedGraha(
                    Graha.of(c.graha(i)), c.longitudeDeg(i), c.tropicalDeg(i), c.latitudeDeg(i), c.distanceAu(i),
                    c.speedDegPerDay(i),
                    new Placement(c.houseBhava(i), HouseSystem.of(c.houseMethod(i)), c.houseThrough(i),
                            c.houseFromMadhyaDeg(i)),
                    new Placement(c.placementBhava(i), HouseSystem.of(c.placementMethod(i)), c.placementThrough(i),
                            c.placementFromMadhyaDeg(i))));
        }
        return List.copyOf(out);
    }

    /** The twelve bhavas for "which house is it in", first to twelfth. Ports {@code Chart.houses}. */
    static List<Bhava> houses(Chart chart) {
        Charts.Houses c = decoded(chart).houses();
        int base = chart.index() * 12;
        List<Bhava> out = new ArrayList<>(12);
        for (int i = base; i < base + 12; i += 1) {
            out.add(new Bhava(c.madhyaDeg(i), c.sandhiDeg(i)));
        }
        return List.copyOf(out);
    }

    /** The twelve bhavas of the chart's chalit. Ports {@code Chart.chalit}. */
    static List<Bhava> chalit(Chart chart) {
        Charts.Chalit c = decoded(chart).chalit();
        int base = chart.index() * 12;
        List<Bhava> out = new ArrayList<>(12);
        for (int i = base; i < base + 12; i += 1) {
            out.add(new Bhava(c.madhyaDeg(i), c.sandhiDeg(i)));
        }
        return List.copyOf(out);
    }
}
