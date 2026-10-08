package com.teispace.teistro;

/**
 * The graha entered a nakshatra.
 *
 * @param into The nakshatra entered.
 * @param motion How the graha was moving.
 */
public record NakshatraIngress(
        Nakshatra into,
        Motion motion) implements HitEvent {
    /**
     * Which kind of event this is.
     *
     * @return {@link HitKind#NAKSHATRA_INGRESS}
     */
    @Override
    public HitKind kind() {
        return HitKind.NAKSHATRA_INGRESS;
    }
}
