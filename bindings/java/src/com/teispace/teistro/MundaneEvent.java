package com.teispace.teistro;

import java.util.Objects;

/**
 * An event on record, cast as a chart of {@link ChartKind#EVENT}
 * ({@code 03-design/mundane-charts.md}).
 *
 * @param name what happened
 * @param instant when, a Julian day (UTC)
 * @param source where the instant is recorded, carried on the answer
 */
public record MundaneEvent(String name, double instant, String source) {
    /** Keeps the name and the source present. */
    public MundaneEvent {
        Objects.requireNonNull(name, "name");
        Objects.requireNonNull(source, "source");
    }
}
