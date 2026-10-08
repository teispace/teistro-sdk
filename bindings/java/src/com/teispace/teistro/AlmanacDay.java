package com.teispace.teistro;

import java.util.List;
import java.util.Optional;
import java.util.function.IntFunction;
import java.util.function.IntToDoubleFunction;
import java.util.function.IntUnaryOperator;

import com.teispace.teistro.blob.Panchanga;
import com.teispace.teistro.record.Provenance;

/** One day of an almanac: a view over its batch, not a copy. */
public final class AlmanacDay {
    private final Almanac batch;
    private final int index;

    AlmanacDay(Almanac batch, int index) {
        this.batch = batch;
        this.index = index;
    }

    /**
     * The batch this day belongs to.
     *
     * @return the batch
     */
    public Almanac batch() {
        return batch;
    }

    /**
     * Where in that batch it sits.
     *
     * @return the index, from 0
     */
    public int index() {
        return index;
    }

    private Panchanga.Days days() {
        return batch.decoded().days();
    }

    /**
     * What computed this day, and under what: the batch's provenance stamped
     * with this day's own content hash.
     *
     * @return the provenance
     */
    public Provenance provenance() {
        Provenance p = batch.provenance();
        String own = batch.decoded().contentHashes().substring(64 * index, 64 * index + 64);
        return new Provenance(p.appliedConventions(), p.calculationVersion(), p.calendar(), p.catalogueVersion(),
                p.confidence(), own, p.deviation(), p.fallbacksUsed(), p.inputHash(), p.moduleVersions(),
                p.packs(), p.profile(), p.provider(), p.sdkVersion(), p.settingsHash(), p.time(), p.warnings());
    }

    /**
     * The day itself: its date, weekday and sunrises.
     *
     * @return the day
     */
    public LocalDay day() {
        return VedicReads.localDay(batch.decoded().day(), index);
    }

    /**
     * What the spans are clipped to.
     *
     * @return the window
     */
    public Interval window() {
        return new Interval(days().windowFrom(index), days().windowTo(index));
    }

    /**
     * The lunar month, under both conventions.
     *
     * @return the month
     */
    public Month month() {
        Panchanga.Days days = days();
        return new Month(Masa.of(days.month(index)), Masa.of(days.amanta(index)), Masa.of(days.purnimanta(index)),
                Paksha.of(days.paksha(index)), LunarMonth.of(batch.decoded().lunarMonth()),
                MonthKind.of(days.monthKind(index)));
    }

    /**
     * Which half of the year the day falls in.
     *
     * @return the ayana
     */
    public Ayana ayana() {
        return Ayana.of(days().ayana(index));
    }

    /**
     * Which season the day falls in, under {@code panchanga.ritu}: by default
     * the season of the sidereal solar month the day belongs to, its first
     * day placed by {@code panchanga.solar_month_start}
     * ({@code 03-design/ritu-measured.md}).
     *
     * @return the ritu
     */
    public Ritu ritu() {
        return Ritu.of(days().ritu(index));
    }

    /**
     * The direction not to travel in, which is the vara's.
     *
     * @return the direction
     */
    public Direction dishaShool() {
        return Direction.of(days().dishaShool(index));
    }

    /**
     * When the Sun entered a new sign inside the day.
     *
     * @return the instant, a Julian day (UTC), if it did
     */
    public Optional<Double> sankranti() {
        Panchanga.Days days = days();
        return days.hasSankranti(index) == 0 ? Optional.empty() : Optional.of(days.sankranti(index));
    }

    /**
     * Abhijit; empty on a day with no daylight.
     *
     * @return abhijit, if the day has one
     */
    public Optional<Abhijit> abhijit() {
        Panchanga.Days days = days();
        if (days.hasAbhijit(index) == 0) {
            return Optional.empty();
        }
        return Optional.of(new Abhijit(new Interval(days.abhijitFrom(index), days.abhijitTo(index)),
                days.abhijitEffective(index) != 0));
    }

    /**
     * Brahma muhurta; empty when the night before is not known.
     *
     * @return its interval, if known
     */
    public Optional<Interval> brahma() {
        Panchanga.Days days = days();
        if (days.hasBrahma(index) == 0) {
            return Optional.empty();
        }
        return Optional.of(new Interval(days.brahmaFrom(index), days.brahmaTo(index)));
    }

    /**
     * The tithis that touch the day.
     *
     * @return the spans
     */
    public List<Span<Tithi>> tithi() {
        Panchanga.Tithi c = batch.decoded().tithi();
        return spans("tithi", new SpanColumns(c::member, c::wholeFrom, c::wholeTo, c::insideFrom, c::insideTo,
                c::sunrises, c::endsGhati, c::endsPala, c::endsVipala), Tithi::of);
    }

    /**
     * The nakshatras the Moon was in.
     *
     * @return the spans
     */
    public List<Span<Nakshatra>> nakshatra() {
        Panchanga.Nakshatra c = batch.decoded().nakshatra();
        return spans("nakshatra", new SpanColumns(c::member, c::wholeFrom, c::wholeTo, c::insideFrom, c::insideTo,
                c::sunrises, c::endsGhati, c::endsPala, c::endsVipala), Nakshatra::of);
    }

    /**
     * The nitya yogas.
     *
     * @return the spans
     */
    public List<Span<Yoga>> yoga() {
        Panchanga.Yoga c = batch.decoded().yoga();
        return spans("yoga", new SpanColumns(c::member, c::wholeFrom, c::wholeTo, c::insideFrom, c::insideTo,
                c::sunrises, c::endsGhati, c::endsPala, c::endsVipala), Yoga::of);
    }

    /**
     * The karanas: half-tithis, so three or four on an ordinary day.
     *
     * @return the spans
     */
    public List<Span<Karana>> karana() {
        Panchanga.Karana c = batch.decoded().karana();
        return spans("karana", new SpanColumns(c::member, c::wholeFrom, c::wholeTo, c::insideFrom, c::insideTo,
                c::sunrises, c::endsGhati, c::endsPala, c::endsVipala), Karana::of);
    }

    /**
     * Panchaka, while the Moon is in the last five nakshatras.
     *
     * @return the spans
     */
    public List<Span<Panchaka>> panchaka() {
        Panchanga.Panchaka c = batch.decoded().panchaka();
        return spans("panchaka", new SpanColumns(c::member, c::wholeFrom, c::wholeTo, c::insideFrom, c::insideTo,
                c::sunrises, c::endsGhati, c::endsPala, c::endsVipala), Panchaka::of);
    }

    /**
     * The signs the Moon stood in.
     *
     * @return the spans
     */
    public List<Span<Rashi>> moonSigns() {
        Panchanga.MoonSigns c = batch.decoded().moonSigns();
        return spans("moon_signs", new SpanColumns(c::member, c::wholeFrom, c::wholeTo, c::insideFrom, c::insideTo,
                c::sunrises, c::endsGhati, c::endsPala, c::endsVipala), Rashi::of);
    }

    /**
     * The signs the Sun stood in; two only on a sankranti day.
     *
     * @return the spans
     */
    public List<Span<Rashi>> sunSigns() {
        Panchanga.SunSigns c = batch.decoded().sunSigns();
        return spans("sun_signs", new SpanColumns(c::member, c::wholeFrom, c::wholeTo, c::insideFrom, c::insideTo,
                c::sunrises, c::endsGhati, c::endsPala, c::endsVipala), Rashi::of);
    }

    /**
     * The inauspicious eighths of the daylight.
     *
     * @return the periods
     */
    public List<KaalaPeriod> kaalas() {
        Panchanga.Kaalas c = batch.decoded().kaalas();
        return rows("kaalas", i -> new KaalaPeriod(Kaala.of(c.kaala(i)), new Interval(c.from(i), c.to(i))));
    }

    /**
     * Eight choghadiya of the daylight and eight of the night.
     *
     * @return the periods
     */
    public List<ChoghadiyaPeriod> choghadiya() {
        Panchanga.Choghadiya c = batch.decoded().choghadiya();
        return rows("choghadiya", i -> new ChoghadiyaPeriod(Choghadiya.of(c.choghadiya(i)), Graha.of(c.lord(i)),
                new Interval(c.from(i), c.to(i)), c.daytime(i) != 0));
    }

    /**
     * The twenty-four horas, from sunrise.
     *
     * @return the horas
     */
    public List<Hora> horas() {
        Panchanga.Horas c = batch.decoded().horas();
        return rows("horas", i -> new Hora(c.number(i), Graha.of(c.lord(i)), c.start(i), c.end(i)));
    }

    /**
     * The thirty muhurtas: fifteen of the daylight, then of the night.
     *
     * @return the muhurtas
     */
    public List<Muhurta> muhurtas() {
        Panchanga.Muhurtas c = batch.decoded().muhurtas();
        return rows("muhurtas", i -> new Muhurta(new Interval(c.from(i), c.to(i)), c.daylight(i) != 0));
    }

    /**
     * Every moonrise and moonset inside the day's moon window.
     *
     * @return the events
     */
    public List<MoonEventMoment> moonEvents() {
        Panchanga.MoonEvents c = batch.decoded().moonEvents();
        return rows("moon_events", i -> new MoonEventMoment(c.kind(i) == 0, c.instant(i)));
    }

    /**
     * The muhurta yogas that held, with what made each hold.
     *
     * @return the yogas
     */
    public List<HeldYoga> muhurtaYogas() {
        Panchanga.MuhurtaYogas c = batch.decoded().muhurtaYogas();
        // A VARA_NAKSHATRA cause has no tithi, and the blob leaves the column
        // at nought rather than at a tithi that did not make it.
        return rows("muhurta_yogas", i -> new HeldYoga(MuhurtaYoga.of(c.yoga(i)), new Interval(c.from(i), c.to(i)),
                Vara.of(c.becauseVara(i)), c.becauseKind(i) == 0 ? null : Tithi.of(c.becauseTithi(i)),
                Nakshatra.of(c.becauseNakshatra(i))));
    }

    private <T> List<T> rows(String listName, IntFunction<T> build) {
        int[] range = batch.range(listName, index);
        return Reads.rows(range[0], range[1], build);
    }

    /** The columns every limb's section shares. */
    private record SpanColumns(
            IntUnaryOperator member, IntToDoubleFunction wholeFrom, IntToDoubleFunction wholeTo,
            IntToDoubleFunction insideFrom, IntToDoubleFunction insideTo, IntUnaryOperator sunrises,
            IntUnaryOperator endsGhati, IntUnaryOperator endsPala, IntUnaryOperator endsVipala) {}

    private <T> List<Span<T>> spans(String listName, SpanColumns c, IntFunction<T> member) {
        return rows(listName, i -> new Span<T>(
                member.apply(c.member().applyAsInt(i)),
                new Interval(c.wholeFrom().applyAsDouble(i), c.wholeTo().applyAsDouble(i)),
                new Interval(c.insideFrom().applyAsDouble(i), c.insideTo().applyAsDouble(i)),
                Sunrises.of(c.sunrises().applyAsInt(i)),
                new GhatiPala(c.endsGhati().applyAsInt(i), c.endsPala().applyAsInt(i), c.endsVipala().applyAsInt(i))));
    }
}
