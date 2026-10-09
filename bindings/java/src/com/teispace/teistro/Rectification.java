package com.teispace.teistro;

import java.util.Optional;

/**
 * A chart read as a birth time to rectify, the chart's instant the time on record
 * ({@code 03-design/rectification.md}): each reading the request asked for, and empty for one it
 * did not.
 *
 * @param purified What the purifier leaves standing of the window; empty unless {@code purify}
 *     was asked.
 * @param conception The conception reports at the chart's instant; empty unless
 *     {@code conception} was asked.
 * @param circumstance The circumstances at the chart's instant; empty unless
 *     {@code circumstance} was asked.
 * @param baseline The baseline engine's cascade around it; empty unless {@code baseline} was
 *     asked.
 */
public record Rectification(
        Optional<Purified> purified,
        Optional<Conception> conception,
        Optional<Circumstance> circumstance,
        Optional<BaselineRectification> baseline) {
}
