package com.teispace.teistro;

import java.util.List;
import java.util.Map;
import java.util.Optional;

import com.teispace.teistro.blob.Charts;
import com.teispace.teistro.record.Provenance;

/**
 * One founded chart: a view over its batch, not a copy. Every accessor reads
 * the batch's columns at this chart's index, so a chart costs nothing until
 * something is asked of it.
 */
public final class Chart {
    private final ChartBatch batch;
    private final int index;

    Chart(ChartBatch batch, int index) {
        this.batch = batch;
        this.index = index;
    }

    /**
     * The batch this chart belongs to.
     *
     * @return the batch
     */
    public ChartBatch batch() {
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

    private Charts decoded() {
        return batch.decoded();
    }

    /**
     * What computed this chart, and under what: the batch's provenance stamped
     * with this chart's own content hash, so a chart founded alone carries the
     * hash of its own value and not of a list of one.
     *
     * @return the provenance
     */
    public Provenance provenance() {
        Provenance p = batch.provenance();
        String hashes = decoded().contentHashes();
        String own = hashes.substring(64 * index, 64 * index + 64);
        return new Provenance(p.appliedConventions(), p.calculationVersion(), p.calendar(), p.catalogueVersion(),
                p.confidence(), own, p.deviation(), p.fallbacksUsed(), p.inputHash(), p.moduleVersions(),
                p.packs(), p.profile(), p.provider(), p.sdkVersion(), p.settingsHash(), p.time(), p.warnings());
    }

    /**
     * The instant the chart is cast for.
     *
     * @return a UTC Julian day
     */
    public double instant() {
        return decoded().cast().instant(index);
    }

    /**
     * The lagna at the instant, in the chart's zodiac.
     *
     * @return degrees
     */
    public double lagnaDeg() {
        return decoded().cast().lagnaDeg(index);
    }

    /**
     * The lagna at the sunrise that opened the day.
     *
     * @return degrees
     */
    public double dayLagnaDeg() {
        return decoded().cast().dayLagnaDeg(index);
    }

    /**
     * The ayanamsha applied at this instant; zero if tropical.
     *
     * @return degrees
     */
    public double ayanamshaOffsetDeg() {
        return decoded().cast().ayanamshaOffsetDeg(index);
    }

    /**
     * The catalogued ayanamsha this chart was read under: none for a tropical
     * chart, or one whose settings define their own, which
     * {@link #ayanamshaCustom()} says.
     *
     * @return the ayanamsha, if a catalogued one was applied
     */
    public Optional<Ayanamsha> ayanamsha() {
        return decoded().ayanamshaKind() == 1 ? Optional.of(Ayanamsha.of(decoded().ayanamsha())) : Optional.empty();
    }

    /**
     * Whether the ayanamsha is one the settings define rather than a catalogued one.
     *
     * @return true for the settings' own
     */
    public boolean ayanamshaCustom() {
        return decoded().ayanamshaKind() == 2;
    }

    /**
     * Which arc of its day the instant falls in.
     *
     * @return the arc
     */
    public DayPart dayPart() {
        return DayPart.of(decoded().cast().dayPart(index));
    }

    /**
     * How far through that arc the instant is.
     *
     * @return 0 to 1
     */
    public double dayElapsed() {
        return decoded().cast().dayElapsed(index);
    }

    /**
     * What kind of chart this is.
     *
     * @return the kind
     */
    public ChartKind kind() {
        return batch.kind();
    }

    /**
     * The day the instant belongs to: its sunrise, sunset, next sunrise and weekday.
     *
     * @return the value
     */
    public LocalDay day() {
        return VedicReads.day(this);
    }

    /**
     * How far into its day the instant is: the ishtakaal and the hora.
     *
     * @return the value
     */
    public ChartTiming timing() {
        return VedicReads.timing(this);
    }

    /**
     * Each graha's state: dignity, friendship, combustion, ages, war and avasthas.
     *
     * @return the rows
     */
    public List<GrahaState> states() {
        return VedicReads.states(this);
    }

    /**
     * The houses as the chart's house system serves them.
     *
     * @return the rows
     */
    public List<ServiceBhava> bhavas() {
        return VedicReads.bhavas(this);
    }

    /**
     * The derived points: the Sun's five, Gulika and Mandi, the special lagnas, the Yogi points.
     *
     * @return the rows
     */
    public List<DerivedPoint> points() {
        return VedicReads.points(this);
    }

    /**
     * The Tajika sahams the request named.
     *
     * @return the rows
     */
    public List<TajikaSaham> sahams() {
        return TajikaReads.sahams(this);
    }

    /**
     * The annual charts' instants the request asked for.
     *
     * @return the rows
     */
    public List<Pravesha> praveshas() {
        return TajikaReads.praveshas(this);
    }

    /**
     * Which bodies reach which: the graha drishti and the rashi drishti.
     *
     * @return the rows
     */
    public List<Drishti> aspects() {
        return VedicReads.aspects(this);
    }

    /**
     * The Ashtakavarga, when it was asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<Ashtakavarga> ashtakavarga() {
        return VedicReads.ashtakavarga(this);
    }

    /**
     * The Bhava bala, when it was asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<BhavaBala> bhavaBala() {
        return VedicReads.bhavaBala(this);
    }

    /**
     * The Shadbala, when it was asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<Shadbala> shadbala() {
        return VedicReads.shadbala(this);
    }

    /**
     * The dasha phala reading, when it was asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<DashaPhalaReading> dashaPhala() {
        return VedicReads.dashaPhala(this);
    }

    /**
     * The transit hit list over the window asked for.
     *
     * @return the rows
     */
    public List<Hit> hits() {
        return VedicReads.hits(this);
    }

    /**
     * Saturn's periods from the Moon over the window asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<SadeSatiReport> sadeSati() {
        return VedicReads.sadeSati(this);
    }

    /**
     * The Krishnamurti Paddhati reading, when it was asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<KpReading> kp() {
        return KpReads.kp(this);
    }

    /**
     * The prashna reading, when a question was put.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<Prashna> prashna() {
        return KpReads.prashna(this);
    }

    /**
     * The remedies reading, when it was asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<Remedies> remedies() {
        return KpReads.remedies(this);
    }

    /**
     * The chart read as Lal Kitab reads it, when it was asked for: the teva's planets, houses,
     * artificial planets, debts and conditions, the 35-year cycle's periods and, when a year was
     * named, its ruler, thirds and annual teva ({@code 03-design/lalkitab.md}).
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<LalKitab> lalkitab() {
        return LalKitabReads.lalkitab(this);
    }

    /**
     * The rectification readings, when they were asked for: each member
     * empty unless the request named it.
     *
     * @return the readings, empty when they were not asked for
     */
    public Optional<Rectification> rectification() {
        return KpReads.rectification(this);
    }

    /**
     * The Ashta Koota against the partner, when one was named.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<AshtaKoota> matching() {
        return MatchReads.matching(this);
    }

    /**
     * The ten poruthams against the partner, when one was named.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<Porutham> porutham() {
        return MatchReads.porutham(this);
    }

    /**
     * The Kuja dosha of both sides, when a partner was named.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<Kuja> kuja() {
        return MatchReads.kuja(this);
    }

    /**
     * The marriage doshas, when they were asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<List<MarriageDosha>> marriageDoshas() {
        return MatchReads.marriageDoshas(this);
    }

    /**
     * The gochar readings over the instants asked for.
     *
     * @return the rows
     */
    public List<GocharReading> gochar() {
        return VedicReads.gochar(this);
    }

    /**
     * The Jaimini reading, when it was asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<JaiminiReading> jaimini() {
        return VedicReads.jaimini(this);
    }

    /**
     * The Avakahada chakra, when it was asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<Avakahada> avakahada() {
        return VedicReads.avakahada(this);
    }

    /**
     * The Vaiseshikamsa standing, when it was asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<VaiseshikamsaReading> vaiseshikamsa() {
        return VedicReads.vaiseshikamsa(this);
    }

    /**
     * The Vimshopaka bala, when it was asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<Vimshopaka> vimshopaka() {
        return VedicReads.vimshopaka(this);
    }

    /**
     * The dashas the request named.
     *
     * @return the rows
     */
    public List<Dasha> dashas() {
        return DashaReads.dashas(this);
    }

    /**
     * What the chart answers by rule, as JSON, when rules were asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<Map<String, Object>> rules() {
        return VedicReads.rules(this);
    }

    /**
     * The narrative plans, as JSON, when they were asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<Map<String, Object>> plans() {
        return VedicReads.plans(this);
    }

    /**
     * The chart drawings the request named.
     *
     * @return the rows
     */
    public List<Drawing> drawings() {
        return VedicReads.drawings(this);
    }

    /**
     * The divisional charts the request named.
     *
     * @return the rows
     */
    public List<VargaChart> vargas() {
        return VedicReads.vargas(this);
    }

    /**
     * The nine grahas' placements.
     *
     * @return the rows
     */
    public List<PlacedGraha> grahas() {
        return VedicReads.grahas(this);
    }

    /**
     * The outer planets' placements, when they were asked for.
     *
     * @return the rows
     */
    public List<PlacedGraha> outer() {
        return VedicReads.outer(this);
    }

    /**
     * The twelve bhavas of the placement system.
     *
     * @return the rows
     */
    public List<Bhava> houses() {
        return VedicReads.houses(this);
    }

    /**
     * The twelve bhavas of the chalit system.
     *
     * @return the rows
     */
    public List<Bhava> chalit() {
        return VedicReads.chalit(this);
    }

    /**
     * The essential dignities, when they were asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<Dignities> dignities() {
        return WesternReads.dignities(this);
    }

    /**
     * The accidental fortitudes, when they were asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<Fortitudes> fortitudes() {
        return WesternReads.fortitudes(this);
    }

    /**
     * The lots, when they were asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<Lots> lots() {
        return WesternReads.lots(this);
    }

    /**
     * The considerations before judgement, when they were asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<Considerations> considerations() {
        return WesternReads.considerations(this);
    }

    /**
     * How the matter asked of the chart perfects, when one was asked.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<Matter> perfection() {
        return WesternReads.perfection(this);
    }

    /**
     * The progressions and directions, when they were asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<Progressions> progressions() {
        return WesternReads.progressions(this);
    }

    /**
     * The Western aspects, when they were asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<List<WesternAspectRow>> westernAspects() {
        return WesternReads.westernAspects(this);
    }

    /**
     * The declinations, when they were asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<Declinations> declinations() {
        return WesternReads.declinations(this);
    }

    /**
     * The parallels and contra-parallels, when they were asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<List<ParallelRow>> parallels() {
        return WesternReads.parallels(this);
    }

    /**
     * The antiscia, when they were asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<Antiscia> antiscia() {
        return WesternReads.antiscia(this);
    }

    /**
     * The harmonic chart, when one was asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<HarmonicChart> harmonic() {
        return WesternReads.harmonic(this);
    }

    /**
     * The Western house placements, when they were asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<WesternHouses> westernHouses() {
        return WesternReads.westernHouses(this);
    }

    /**
     * The midpoints, when they were asked for.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<List<MidpointRow>> midpoints() {
        return WesternReads.midpoints(this);
    }

    /**
     * The synastry aspects against the partner, when one was named.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<List<SynastryRow>> synastry() {
        return WesternReads.synastry(this);
    }

    /**
     * The synastry parallels, when a partner was named.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<List<SynastryParallelRow>> synastryParallels() {
        return WesternReads.synastryParallels(this);
    }

    /**
     * The synastry antiscia, when a partner was named.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<List<AntiscionRow>> synastryAntiscia() {
        return WesternReads.synastryAntiscia(this);
    }

    /**
     * The synastry midpoints, when a partner was named.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<List<SynastryMidpointRow>> synastryMidpoints() {
        return WesternReads.synastryMidpoints(this);
    }

    /**
     * The composite chart, when a partner was named.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<Composite> synastryComposite() {
        return WesternReads.synastryComposite(this);
    }

    /**
     * The Davison birth, when a partner was named.
     *
     * @return the reading, empty when it was not asked for
     */
    public Optional<DavisonBirth> synastryDavison() {
        return WesternReads.synastryDavison(this);
    }
}
