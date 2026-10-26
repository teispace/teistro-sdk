package com.teispace.teistro;

import java.util.Optional;

/**
 * What a chart's instant falls in: the sign rising and its span, which BPHS ch. 92 v. 4's lagna
 * gandanta reads, the eclipse seen where the chart is cast, which Brihat Jataka ch. 6 v. 9 reads
 * (C378), and the Moon's stay in its nakshatra.
 *
 * <pre>{@code
 * Chart chart = ctx.chart().found(instant, place, offset, ChartOptions.builder().moment(true).build());
 * Rashi rising = chart.moment().orElseThrow().rising().sign();
 * }</pre>
 *
 * @param rising the sign rising and when it rose and sets
 * @param eclipse the eclipse the instant falls in
 * @param nakshatra the Moon's stay in its nakshatra; empty on a tropical chart, where a nakshatra
 *     means nothing
 */
public record Moment(RisingSign rising, MomentEclipse eclipse, Optional<MoonStay> nakshatra) {}
