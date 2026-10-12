package com.teispace.teistro;

import java.util.List;

/**
 * The Bhuvanapradipa's graded yogas (n178, C373): every clause the grades are built from, and the
 * grades whose clauses hold, by name, never ranked.
 *
 * @param rule The aspect the clauses were read by: {@code PARASHARI} or {@code TAJIKA}.
 * @param lords The lords' configurations that hold ({@code OWN_HOUSES}, {@code
 *     EACH_OTHERS_HOUSES}, {@code EACH_OTHER}); empty when no house was asked.
 * @param moonSeesLagna Whether the Moon looks at the lagna.
 * @param lagneshaSeesLagna Whether the lagnesha looks at the lagna.
 * @param onLagna The benefics, the lagnesha left out, looking at the lagna.
 * @param onLagnesha The benefics looking at the lagnesha.
 * @param moonFree Whether no malefic looks at the Moon.
 * @param grades The grades whose clauses hold ({@code PARIPURNA}, {@code PADA}, {@code ARDHA},
 *     {@code PADONA}, {@code TRIBHAGONA}, {@code PURNA}), in the passage's order.
 * @param ill The grahas that give ill: sharing a sign with a malefic, looked at by one, or burnt
 *     by the Sun.
 */
public record PrashnaGraded(
        String rule,
        List<String> lords,
        boolean moonSeesLagna,
        boolean lagneshaSeesLagna,
        List<Graha> onLagna,
        List<Graha> onLagnesha,
        boolean moonFree,
        List<String> grades,
        List<Graha> ill) {
    /** The value, its lists copied and unmodifiable. */
    public PrashnaGraded {
        lords = List.copyOf(lords);
        onLagna = List.copyOf(onLagna);
        onLagnesha = List.copyOf(onLagnesha);
        grades = List.copyOf(grades);
        ill = List.copyOf(ill);
    }
}
