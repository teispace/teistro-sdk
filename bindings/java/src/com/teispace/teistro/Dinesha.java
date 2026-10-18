package com.teispace.teistro;

import com.teispace.teistro.request.MaseshaReading;

/**
 * The lord of one day, and the chain it was read beside ({@code 03-design/varshesha.md}, "The
 * day's lord").
 *
 * @param graha The lord under {@link #reading()}.
 * @param reading The reading that made it lord: the chain, or the day lagna's lord.
 * @param bearers The seven it was chosen among.
 * @param chain The year lord's chain over the seven, carried under either reading.
 */
public record Dinesha(Graha graha, MaseshaReading reading, DayBearers bearers, YearLord chain) {
}
