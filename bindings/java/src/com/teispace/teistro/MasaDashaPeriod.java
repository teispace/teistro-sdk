package com.teispace.teistro;

/**
 * One Hudda lord's run of a month.
 *
 * @param lord The Hudda's lord.
 * @param degrees The Hudda's width, degrees: the solar days it runs.
 * @param starts When it begins, a Julian day (UTC).
 * @param ends When it ends, a Julian day (UTC): where the next begins.
 */
public record MasaDashaPeriod(Graha lord, int degrees, double starts, double ends) {
}
