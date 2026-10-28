package com.teispace.teistro;

import java.util.List;
import java.util.Map;

/**
 * One cell of the śatapada cakra as a name may begin with it (C297, C770,
 * C771).
 *
 * @param cell its place among the cakra's 112 cells, 0 for a, Krittika's first
 * @param nakshatra its star; may be null, for Abhijit's row
 * @param pada its pada, 1 to 4; may be null, for Abhijit's, which is no pada (C298)
 * @param rashi the sign it stands in; Makara for Abhijit's (C771)
 * @param devanagari the print's spelling (C299)
 * @param iast its IAST
 * @param varga the varga of its letter (C295)
 * @param initials every letter that reaches the cell, its own first
 */
public record PadaSyllable(int cell, Nakshatra nakshatra, Integer pada, Rashi rashi, String devanagari, String iast,
        NameVarga varga, List<NameInitial> initials) {
    static PadaSyllable read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        return new PadaSyllable(
                Reads.integer(o, "cell"),
                Naming.nakshatra(Reads.field(o, "nakshatra")),
                Reads.optionalInteger(o, "pada"),
                Naming.rashi(Reads.field(o, "rashi")),
                Reads.string(o, "devanagari"),
                Reads.string(o, "iast"),
                Naming.varga(Reads.field(o, "varga")),
                Reads.each(o, "initials", NameInitial::read));
    }
}
