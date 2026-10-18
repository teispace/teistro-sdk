package com.teispace.teistro;

import com.teispace.teistro.request.MaseshaReading;

/**
 * The lord of one month, and the chain it was read beside ({@code 03-design/varshesha.md}, "The
 * month's lord").
 *
 * @param graha The lord under {@link #reading()}.
 * @param reading The reading that made it lord.
 * @param bearers The six it was chosen among.
 * @param chain The year lord's chain over the six, carried under either reading so a reader sees
 *     where the two part.
 */
public record Masesha(Graha graha, MaseshaReading reading, MonthBearers bearers, YearLord chain) {
}
