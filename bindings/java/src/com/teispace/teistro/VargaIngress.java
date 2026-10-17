package com.teispace.teistro;

/**
 * The graha entered a new sign of a divisional chart (C418).
 *
 * @param varga The divisional chart whose sign changed.
 * @param into The sign entered in that chart.
 * @param motion How the graha was moving.
 */
public record VargaIngress(
        Varga varga,
        Rashi into,
        Motion motion) implements HitEvent {
    /**
     * Which kind of event this is.
     *
     * @return {@link HitKind#VARGA_INGRESS}
     */
    @Override
    public HitKind kind() {
        return HitKind.VARGA_INGRESS;
    }
}
