package com.teispace.teistro;

/**
 * A point of the sky placed in a camera's picture ({@code 03-design/sky-pose.md}).
 *
 * @param inView whether it is in front of the camera and inside the frame, edges included
 * @param x across the picture, 0 at its left edge and 1 at its right; outside [0, 1] beyond them, null behind the
 *     camera
 * @param y down the picture, 0 at its top edge and 1 at its bottom; outside [0, 1] beyond them, null behind the
 *     camera
 * @param offAxisDeg its angle from the line of sight, degrees, 0 to 180
 */
public record ScreenPlacement(boolean inView, Double x, Double y, double offAxisDeg) {}
