package com.teispace.teistro;

import java.util.List;

/**
 * One house of the day, read through its navamsha lord ({@code 03-design/varshesha.md}, "The
 * day's results").
 *
 * @param house The house, 1 to 12 by whole signs from the day lagna.
 * @param amshaLord The lord of the navamsha the house's point falls in.
 * @param clauses The clauses of vv. 20–21 that hold, in the verses' order.
 */
public record DayHouse(int house, Graha amshaLord, List<DayHouseClause> clauses) {
}
