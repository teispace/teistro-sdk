package com.teispace.teistro;

/**
 * The settings the ruling planets were read under: {@code count} ({@code FIVE} or
 * {@code WITH_SUBS}, C150), {@code nodeRulers} ({@code SIGN_OR_CONJOINED} or {@code SIGN}, C152)
 * and {@code retrogradeRejection} ({@code STAR} or {@code STAR_OR_SUB}, C153).
 *
 * @param count How many rulers are counted.
 * @param nodeRulers How a node stands for a ruler.
 * @param retrogradeRejection How a retrograde planet rejects.
 */
public record KpRulingRules(
        String count,
        String nodeRulers,
        String retrogradeRejection) {
}
