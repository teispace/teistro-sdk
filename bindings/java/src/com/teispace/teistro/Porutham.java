package com.teispace.teistro;

import java.util.List;

/**
 * The ten considerations of a bride and a groom (<i>Kalaprakasika</i> XIII). Never a verdict: "at
 * least five" is the reader's to apply.
 *
 * @param considerations The ten, in the chapter's order.
 * @param agreeing How many agree.
 * @param chiefAgreeing How many of the chief five agree: Dhinam, Ganam, Yoni, Rasi and Rajju.
 * @param exception The p. 76 exception's clauses.
 */
public record Porutham(
        List<PoruthamRow> considerations,
        int agreeing,
        int chiefAgreeing,
        PoruthamException exception) {
    /** The value, its lists copied and unmodifiable. */
    public Porutham {
        considerations = List.copyOf(considerations);
    }
}
