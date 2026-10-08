package com.teispace.teistro;

/**
 * Why a planet is a ruling planet: {@code kind} is {@code LAGNA_STAR}, {@code LAGNA_SIGN},
 * {@code LAGNA_SUB}, {@code MOON_STAR}, {@code MOON_SIGN}, {@code MOON_SUB}, {@code DAY_LORD} or
 * {@code AGENT}, a node standing for the ruler {@code of}, {@code by} being {@code IN_ITS_SIGN} or
 * {@code CONJOINED} (C152).
 *
 * @param kind The reason.
 * @param of The ruler an {@code AGENT} node stands for; null otherwise. May be null.
 * @param by How an {@code AGENT} node stands for it; null otherwise. May be null.
 */
public record KpReason(
        String kind,
        Graha of,
        String by) {
}
