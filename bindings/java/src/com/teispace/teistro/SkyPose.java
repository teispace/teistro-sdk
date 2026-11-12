package com.teispace.teistro;

import java.util.Map;
import java.util.Optional;

import com.teispace.teistro.request.DevicePose;

/**
 * A sky view seen through a device's camera ({@code 03-design/sky-pose.md}): the sky unchanged, the pose, and
 * where each of the sky's points falls in the picture.
 *
 * @param sky the sky view, unchanged
 * @param pose the pose it is seen in, its azimuth in [0, 360)
 * @param verticalFovDeg the field of view top to bottom, degrees
 * @param screen one placement for each point of {@code sky}'s list of the same name
 */
public record SkyPose(SkyView sky, DevicePose pose, double verticalFovDeg, SkyScreen screen) {
    /**
     * The rising lagna point, the ecliptic's {@code ASCENDANT}, with its placement.
     *
     * <pre>{@code
     * boolean risingInView = seen.ascendant().map(rising -> rising.getValue().inView()).orElse(false);
     * }</pre>
     *
     * @return the point and its placement, or empty at the one instant the ecliptic lies in the horizon
     */
    public Optional<Map.Entry<EclipticPoint, ScreenPlacement>> ascendant() {
        for (int at = 0; at < sky.ecliptic().size(); at++) {
            EclipticPoint point = sky.ecliptic().get(at);
            if ("ASCENDANT".equals(point.kind())) {
                return Optional.of(Map.entry(point, screen.ecliptic().get(at)));
            }
        }
        return Optional.empty();
    }
}
