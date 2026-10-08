package com.teispace.teistro;

import java.util.Objects;

/**
 * The columns a provider answers with, each {@link PositionQuery#cellCount}
 * long. A column left null is zeroes, which is what a provider that
 * computes no speeds means.
 *
 * <p><b>Answering at all asserts the answer is in the frame that was asked
 * for.</b> A null {@code frameBits} means "the frame you asked for", not
 * "my own": a provider that computes in one frame compares
 * {@link PositionQuery#frameBits} with its own and answers empty instead,
 * and the SDK then asks again in {@link EphemerisProvider#nativeFrame} and
 * completes the rest. Answering the wrong frame silently is the one mistake
 * this record makes easy, so it is named here.
 *
 * <pre>{@code
 * PositionAnswer.of(lon, lat, dist).withLonSpeed(speeds)
 * }</pre>
 *
 * @param lon the longitudes
 * @param lat the latitudes
 * @param dist the distances
 * @param lonSpeed the longitudes' speeds, or null
 * @param latSpeed the latitudes' speeds, or null
 * @param distSpeed the distances' speeds, or null
 * @param status each cell's {@link ProviderCode} id, or null for all {@code OK}
 * @param source each cell's source bits, or null
 * @param frameBits the frame answered, packed, or null for the frame asked
 */
public record PositionAnswer(
        double[] lon, double[] lat, double[] dist, double[] lonSpeed, double[] latSpeed, double[] distSpeed,
        int[] status, int[] source, Long frameBits) {
    /** The answer, with the three columns every answer has present. */
    public PositionAnswer {
        Objects.requireNonNull(lon, "lon");
        Objects.requireNonNull(lat, "lat");
        Objects.requireNonNull(dist, "dist");
    }

    /**
     * An answer of positions alone.
     *
     * @param lon the longitudes
     * @param lat the latitudes
     * @param dist the distances
     * @return the answer
     */
    public static PositionAnswer of(double[] lon, double[] lat, double[] dist) {
        return new PositionAnswer(lon, lat, dist, null, null, null, null, null, null);
    }

    /**
     * This answer with the longitudes' speeds.
     *
     * @param speeds the speeds
     * @return the answer
     */
    public PositionAnswer withLonSpeed(double[] speeds) {
        return new PositionAnswer(lon, lat, dist, speeds, latSpeed, distSpeed, status, source, frameBits);
    }

    /**
     * This answer with every speed.
     *
     * @param lonSpeed the longitudes' speeds
     * @param latSpeed the latitudes' speeds
     * @param distSpeed the distances' speeds
     * @return the answer
     */
    public PositionAnswer withSpeeds(double[] lonSpeed, double[] latSpeed, double[] distSpeed) {
        return new PositionAnswer(lon, lat, dist, lonSpeed, latSpeed, distSpeed, status, source, frameBits);
    }

    /**
     * This answer with a status per cell.
     *
     * @param status each cell's {@link ProviderCode} id
     * @return the answer
     */
    public PositionAnswer withStatus(int[] status) {
        return new PositionAnswer(lon, lat, dist, lonSpeed, latSpeed, distSpeed, status, source, frameBits);
    }

    /**
     * This answer in a frame other than the one asked for, which the SDK
     * then completes.
     *
     * @param bits the frame answered, packed
     * @return the answer
     */
    public PositionAnswer inFrame(long bits) {
        return new PositionAnswer(lon, lat, dist, lonSpeed, latSpeed, distSpeed, status, source, bits);
    }
}
