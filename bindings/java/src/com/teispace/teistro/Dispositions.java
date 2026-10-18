package com.teispace.teistro;

import java.util.List;

/**
 * A chart's dispositors under the traditional rulers
 * ({@code 03-design/essential-dignities.md} §Dispositors).
 *
 * <pre>{@code
 * Dispositions read = chart.dignities().orElseThrow().dispositions();
 * Graha last = read.finalDispositor(); // null for none
 * }</pre>
 *
 * @param chains one chain a planet, in the order {@link Dignities#planets()} gives them
 * @param ends where the chains stop, each once, in the order the chains first reach it
 * @param finalDispositor the one planet in its own sign every chain stops at; null for more
 *     than one end or a cycle (C415). May be null.
 */
public record Dispositions(List<DispositorChain> chains, List<DispositorEnd> ends, Graha finalDispositor) {
    /** Keeps the lists unmodifiable. */
    public Dispositions {
        chains = List.copyOf(chains);
        ends = List.copyOf(ends);
    }

    /**
     * The rulers the chains were followed under: the dignities' own.
     *
     * @return {@code TRADITIONAL}
     */
    public String rulers() {
        return "TRADITIONAL";
    }
}
