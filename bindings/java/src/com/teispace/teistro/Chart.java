package com.teispace.teistro;

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
}
