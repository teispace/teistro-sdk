package com.teispace.teistro;

import java.util.List;

/**
 * The day's results ({@code 03-design/varshesha.md}, "The day's results"): the clauses that hold,
 * no verdict.
 *
 * @param moonState The Moon's state within her sign.
 * @param houses The twelve houses, first to twelfth.
 * @param lords The four lords, in {@link DayLord}'s order.
 * @param clauses The clauses about the whole chart that hold, in the verses' order.
 */
public record DinaPhala(MoonState moonState, List<DayHouse> houses, List<DayLordPlace> lords,
        List<DayClause> clauses) {
}
