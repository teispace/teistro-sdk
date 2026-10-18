package com.teispace.teistro;

import java.util.List;

/**
 * The month's results ({@code 03-design/varshesha.md}, "The month's results"): the clauses that
 * hold, no verdict.
 *
 * @param houses The twelve houses, first to twelfth.
 * @param lords The five lords, in {@link MonthLord}'s order.
 */
public record MasaPhala(List<MonthHouse> houses, List<MonthLordPlace> lords) {
}
