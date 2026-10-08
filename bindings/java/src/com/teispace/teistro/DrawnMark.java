package com.teispace.teistro;

/**
 * A body drawn at its own degree on a wheel.
 *
 * @param body The body, as a catalogue key.
 * @param ring The ring it is drawn in.
 * @param at Where it is drawn.
 * @param longitudeDeg The longitude that put it there, degrees.
 */
public record DrawnMark(
        String body,
        int ring,
        UnitPoint at,
        double longitudeDeg) {
}
