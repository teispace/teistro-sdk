package com.teispace.teistro;

/**
 * The graha entered a sign; a retrograde ingress enters the one before the line it crossed.
 *
 * @param into The sign entered.
 * @param motion How the graha was moving.
 */
public record SignIngress(
        Rashi into,
        Motion motion) implements HitEvent {
    /**
     * Which kind of event this is.
     *
     * @return {@link HitKind#SIGN_INGRESS}
     */
    @Override
    public HitKind kind() {
        return HitKind.SIGN_INGRESS;
    }
}
