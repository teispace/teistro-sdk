package com.teispace.teistro;

import java.util.List;

/**
 * One house of a month, read through its two planets ({@code 03-design/varshesha.md}, "The
 * month's results").
 *
 * @param house The house, 1 to 12 by whole signs from the month lagna.
 * @param amshaLord The lord of the navamsha the house's point falls in.
 * @param lordAmshaLord The lord of the navamsha the house's sign lord stands in.
 * @param clauses The clauses of vv. 7–11 that hold, in the verses' order.
 * @param seenByAmshaLord The lords of v. 10 the house's navamsha lord sees.
 * @param seenByMoon The lords of v. 10 the Moon sees.
 */
public record MonthHouse(
        int house,
        Graha amshaLord,
        Graha lordAmshaLord,
        List<MonthHouseClause> clauses,
        List<MonthLord> seenByAmshaLord,
        List<MonthLord> seenByMoon) {
}
