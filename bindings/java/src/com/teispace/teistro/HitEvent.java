package com.teispace.teistro;

/**
 * What happened in one event of the transit hit list: a {@code SignIngress}, a
 * {@code NakshatraIngress}, a {@code Station} or an {@code AspectHit} by default, and a
 * {@code VargaIngress}, {@code LunarPhase}, {@code PhaseReturn}, {@code SolarEclipseHit} or
 * {@code LunarEclipseHit} when the request's kinds ask, each saying its {@code kind}.
 */
public sealed interface HitEvent
        permits SignIngress, NakshatraIngress, Station, AspectHit, VargaIngress, LunarPhase,
                PhaseReturn, SolarEclipseHit, LunarEclipseHit {
    /**
     * Which kind of event this is.
     *
     * @return the kind
     */
    HitKind kind();
}
