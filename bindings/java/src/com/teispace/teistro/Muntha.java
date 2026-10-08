package com.teispace.teistro;

/**
 * The Muntha at one return: the birth lagna's sign advanced one sign for each completed year, and
 * that sign's lord.
 *
 * @param sign The sign it has reached; the same under either reading.
 * @param lord The lord of that sign: the Munthesha, first of the annual chart's five
 *     office-bearers and the one that takes the year's lordship when no other qualifies.
 * @param longitudeDeg Its longitude at the return, degrees, under the reading asked for. It
 *     advances thirty degrees over the year.
 */
public record Muntha(
        Rashi sign,
        Graha lord,
        double longitudeDeg) {
}
