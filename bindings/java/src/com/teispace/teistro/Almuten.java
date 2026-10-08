package com.teispace.teistro;

import java.util.ArrayList;
import java.util.List;

/**
 * An almuten as a ranking. Lilly breaks no tie, so every planet holding
 * the greatest total is an almuten (C219).
 *
 * @param totals the seven's totals, in the Chaldean order
 * @param almutens every planet holding the greatest total: one unless they tie
 * @param partakers every planet holding the next total down, Chapter CV's
 *     partakers; empty when all seven tie
 */
public record Almuten(List<AlmutenTotal> totals, List<Graha> almutens, List<Graha> partakers) {
    /** Keeps the lists unmodifiable. */
    public Almuten {
        totals = List.copyOf(totals);
        almutens = List.copyOf(almutens);
        partakers = List.copyOf(partakers);
    }

    /** The ranking of planets by their totals, the two lists in step. */
    static Almuten of(List<Graha> planets, List<Integer> totals) {
        int top = Integer.MIN_VALUE;
        for (int total : totals) {
            top = Math.max(top, total);
        }
        Integer below = null;
        for (int total : totals) {
            if (total < top && (below == null || total > below)) {
                below = total;
            }
        }
        List<AlmutenTotal> ranked = new ArrayList<>();
        for (int at = 0; at < planets.size(); at += 1) {
            ranked.add(new AlmutenTotal(planets.get(at), totals.get(at)));
        }
        return new Almuten(ranked, holding(planets, totals, top),
                below == null ? List.of() : holding(planets, totals, below));
    }

    private static List<Graha> holding(List<Graha> planets, List<Integer> totals, int total) {
        List<Graha> out = new ArrayList<>();
        for (int at = 0; at < planets.size(); at += 1) {
            if (totals.get(at) == total) {
                out.add(planets.get(at));
            }
        }
        return out;
    }
}
