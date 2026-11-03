package com.teispace.teistro;

import com.teispace.teistro.record.Provenance;

/**
 * A mundane moment and the chart cast for it
 * ({@code 03-design/mundane-charts.md}).
 *
 * @param kind which moment
 * @param instant the moment, a Julian day (UTC)
 * @param ingress for a lunation, the ingress it precedes; otherwise null
 * @param solarEclipse for a solar eclipse, its kind; otherwise null
 * @param lunarEclipse for a lunar eclipse, its kind; otherwise null
 * @param name for an event, its name; otherwise null
 * @param source for an event, its source; otherwise null
 * @param chart the chart cast for the moment
 * @param provenance what computed the moment; {@code inputHash} seals the request
 */
public record MundaneChart(
        MundaneKind kind,
        double instant,
        MundaneIngress ingress,
        SolarEclipseKind solarEclipse,
        LunarEclipseKind lunarEclipse,
        String name,
        String source,
        Chart chart,
        Provenance provenance) {
}
