package com.teispace.teistro;

/**
 * What happened in one event of the transit hit list: a {@code SignIngress}, a
 * {@code NakshatraIngress}, a {@code Station} or an {@code AspectHit}, each saying its
 * {@code kind}.
 */
public sealed interface HitEvent
        permits SignIngress, NakshatraIngress, Station, AspectHit {
    /**
     * Which kind of event this is.
     *
     * @return the kind
     */
    HitKind kind();
}
