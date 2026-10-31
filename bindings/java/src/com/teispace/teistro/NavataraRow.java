package com.teispace.teistro;

import com.teispace.teistro.request.Tara;
import java.util.Map;

/**
 * A star of the navatara chakra ({@code 03-design/navatara-chakra.md}).
 *
 * @param nakshatra the star
 * @param count its count from the birth star, 1 (the birth star itself) to 27
 * @param tara the count reduced by nine
 * @param cycle which round of nine the count is in, 1 (janma) to 3 (trijanma)
 * @param special the special star the count names; may be null
 */
public record NavataraRow(Nakshatra nakshatra, int count, Tara tara, int cycle, SpecialStar special) {

    static NavataraRow read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        String tara = Reads.string(o, "tara");
        Object special = Reads.field(o, "special");
        return new NavataraRow(
                Naming.nakshatra(Reads.field(o, "nakshatra")),
                Reads.integer(o, "count"),
                Tara.byKey(tara).orElseThrow(() -> Reads.internal("the library drew a tara this build does not know: " + tara)),
                Reads.integer(o, "cycle"),
                special == null
                        ? null
                        : SpecialStar.byKey(Reads.string(special)).orElseThrow(
                                () -> Reads.internal("the library drew a special star this build does not know: " + special)));
    }
}
