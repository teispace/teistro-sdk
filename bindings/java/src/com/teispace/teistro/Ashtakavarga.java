package com.teispace.teistro;

import java.util.List;

/**
 * A chart's Ashtakavarga: each graha's, the sarvashtakavarga, and their reductions and pindas
 * ({@code 03-design/ashtakavarga-measured.md}).
 *
 * @param shodhana Where the reductions and pindas were made.
 * @param ekadhipatya How a co-ruled sign beside an occupied one was reduced.
 * @param grahas Each graha's, Sun to Saturn.
 * @param sarva The seven grahas' bindus by sign, 337 in all.
 * @param trikona The sum after the trine reduction.
 * @param reduced The sum after both reductions.
 */
public record Ashtakavarga(
        Shodhana shodhana,
        Ekadhipatya ekadhipatya,
        List<GrahaAshtakavarga> grahas,
        List<Integer> sarva,
        List<Integer> trikona,
        List<Integer> reduced) {
    /** The value, its lists copied and unmodifiable. */
    public Ashtakavarga {
        grahas = List.copyOf(grahas);
        sarva = List.copyOf(sarva);
        trikona = List.copyOf(trikona);
        reduced = List.copyOf(reduced);
    }
}
