package com.teispace.teistro;

import java.util.List;
import java.util.function.Supplier;

import com.teispace.teistro.blob.Positions;
import com.teispace.teistro.record.Provenance;
import com.teispace.teistro.record.Step;

/**
 * A decoded positions blob, with the grid read cell by cell. The cells run
 * instants outermost: cell {@code i * bodyCount() + j} is instant {@code i},
 * body {@code j}.
 *
 * <pre>{@code
 * PositionGrid grid = sky.positions(new double[] {2_460_000.5}, List.of(Body.SUN));
 * double sun = grid.at(0, 0).longitude();
 * }</pre>
 */
public final class PositionGrid {
    private final Positions decoded;
    private final Supplier<Provenance> provenance;

    /**
     * A grid over a decoded positions blob.
     *
     * @param decoded the blob as its generated decoder read it
     */
    public PositionGrid(Positions decoded) {
        this.decoded = decoded;
        this.provenance = Lazy.of(() -> Provenance.of(Json.read(decoded.provenanceJson())));
    }

    /**
     * The blob as its generated decoder read it.
     *
     * @return the decoded blob
     */
    public Positions decoded() {
        return decoded;
    }

    /**
     * How many instants the grid covers.
     *
     * @return the count
     */
    public int instantCount() {
        return Math.toIntExact(decoded.jdCount());
    }

    /**
     * How many bodies the grid covers.
     *
     * @return the count
     */
    public int bodyCount() {
        return Math.toIntExact(decoded.bodyCount());
    }

    /**
     * How many cells there are.
     *
     * @return the count
     */
    public int cellCount() {
        return decoded.cells().length();
    }

    /**
     * The scale the instants are on.
     *
     * @return the scale
     */
    public TimeScale timeScale() {
        return TimeScale.of(Math.toIntExact(decoded.scale()));
    }

    /**
     * The bodies, in the order the cells run.
     *
     * @return the bodies
     */
    public List<Body> bodyKeys() {
        Positions.Bodies bodies = decoded.bodies();
        return Reads.rows(0, bodies.length(), at -> Body.of(bodies.body(at)));
    }

    /**
     * The completion steps the SDK applied, in order.
     *
     * @return the steps
     */
    public List<Step> stepsApplied() {
        return Reads.array(Json.read(decoded.steps())).stream().map(Step::of).toList();
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
     * The provenance envelope as the canonical JSON the library stamped: the
     * bytes to store beside the result, byte-identical in every binding.
     *
     * @return the JSON
     */
    public String provenanceJson() {
        return decoded.provenanceJson();
    }

    /**
     * The frame the values are in.
     *
     * @param teistro the library, which unpacks the frame's bits
     * @return the frame
     */
    public Frame frame(Teistro teistro) {
        return teistro.unpackFrame(decoded.frameBits());
    }

    /**
     * One cell of the grid.
     *
     * @param instant the instant's index, from 0
     * @param body the body's index, from 0
     * @return the cell
     * @throws IndexOutOfBoundsException for a cell outside the grid
     */
    public Cell at(int instant, int body) {
        if (instant < 0 || instant >= instantCount() || body < 0 || body >= bodyCount()) {
            throw new IndexOutOfBoundsException(
                    "(" + instant + ", " + body + ") is outside a " + instantCount() + "x" + bodyCount() + " grid");
        }
        int index = instant * bodyCount() + body;
        Positions.Cells cells = decoded.cells();
        return new Cell(cells.lon(index), cells.lat(index), cells.dist(index), cells.lonSpeed(index),
                cells.latSpeed(index), cells.distSpeed(index), cells.status(index), cells.source(index));
    }
}
