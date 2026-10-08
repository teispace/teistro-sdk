package com.teispace.teistro;

import java.util.AbstractList;
import java.util.List;
import java.util.concurrent.ConcurrentHashMap;
import java.util.function.Supplier;

import com.teispace.teistro.blob.Charts;
import com.teispace.teistro.record.Provenance;

/**
 * A batch of charts founded at one place, read one chart at a time. A
 * {@link List} of its charts, so it indexes and iterates; each chart is a
 * view over the batch's columns, not a copy.
 */
public final class ChartBatch extends AbstractList<Chart> {
    private final Charts decoded;
    private final Supplier<Provenance> provenance;
    private final ConcurrentHashMap<String, Object> parsed = new ConcurrentHashMap<>();

    ChartBatch(Charts decoded) {
        this.decoded = decoded;
        this.provenance = Lazy.of(() -> Provenance.of(Json.read(decoded.provenanceJson())));
    }

    /**
     * The blob as its generated decoder read it: every section, every column.
     *
     * @return the decoded blob
     */
    public Charts decoded() {
        return decoded;
    }

    /**
     * What a reader parses from the batch's blob, parsed once however many
     * charts read it: a JSON section's records, a section's rows grouped by
     * chart. {@code name} is the reader's own and says what it holds.
     *
     * <p>Parsed outside the map, so one reader may ask for another's
     * parse; two threads asking at once may both parse, and both get the
     * value the first one kept.
     */
    @SuppressWarnings("unchecked")
    <T> T cached(String name, Supplier<T> parse) {
        Object found = parsed.get(name);
        if (found == null) {
            Object fresh = parse.get();
            found = parsed.putIfAbsent(name, fresh);
            if (found == null) {
                found = fresh;
            }
        }
        return (T) found;
    }

    /**
     * How many charts the batch holds.
     *
     * @return the count
     */
    @Override
    public int size() {
        return (int) decoded.chartCount();
    }

    /**
     * One chart of the batch.
     *
     * @param index where it sits, from 0
     * @return the chart
     * @throws IndexOutOfBoundsException for an index outside the batch
     */
    @Override
    public Chart get(int index) {
        return at(index);
    }

    /**
     * One chart of the batch.
     *
     * @param index where it sits, from 0
     * @return the chart
     * @throws IndexOutOfBoundsException for an index outside the batch
     */
    public Chart at(int index) {
        if (index < 0 || index >= size()) {
            throw new IndexOutOfBoundsException("chart " + index + " is outside a batch of " + size());
        }
        return new Chart(this, index);
    }

    /**
     * What kind of chart these are.
     *
     * @return the kind
     */
    public ChartKind kind() {
        return ChartKind.of(decoded.kind());
    }

    /**
     * The place they were all founded at.
     *
     * @return the place
     */
    public Observer place() {
        return new Observer(new Longitude(decoded.longitudeDeg()), new Latitude(decoded.latitudeDeg()),
                new Altitude(decoded.altitudeM()));
    }

    /**
     * The solar model that reckoned the days, as it describes itself.
     *
     * @return the model
     */
    public String model() {
        return decoded.model();
    }

    /**
     * The completion steps the SDK applied, in order, each {@code name:IMPLEMENTATION}.
     *
     * @return the steps
     */
    public List<String> stepsApplied() {
        return ((List<?>) Json.read(decoded.steps())).stream().map(String::valueOf).toList();
    }

    /**
     * What computed these, and under what; its content hash is the whole batch's.
     *
     * @return the provenance
     */
    public Provenance provenance() {
        return provenance.get();
    }

    /**
     * The provenance as the canonical JSON the library stamped: the bytes to
     * store beside the result, byte-identical in every binding.
     *
     * @return the JSON
     */
    public String provenanceJson() {
        return decoded.provenanceJson();
    }
}
