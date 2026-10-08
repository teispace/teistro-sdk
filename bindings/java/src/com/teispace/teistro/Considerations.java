package com.teispace.teistro;

/**
 * A chart's considerations before judgement, each clause with the facts it
 * rests on and none folded into a verdict
 * ({@code 03-design/hellenistic-considerations.md}).
 *
 * <pre>{@code
 * boolean radical = !chart.considerations().orElseThrow().radicality().grounds().isEmpty();
 * }</pre>
 *
 * @param radicality whether the figure is radical
 * @param ascendant the ascendant's degree
 * @param moon the Moon's place and course
 * @param seventh the seventh house and its lord
 * @param saturnHouse Saturn's house, 1 to 12
 * @param saturnRetrograde whether Saturn is retrograde
 * @param ascendantLordCombust whether the ascendant's lord is combust
 * @param rules the rules they were read under
 */
public record Considerations(
        Radicality radicality, AscendantClause ascendant, MoonClause moon, SeventhClause seventh, int saturnHouse,
        boolean saturnRetrograde, boolean ascendantLordCombust, ConsiderationRules rules) {}
