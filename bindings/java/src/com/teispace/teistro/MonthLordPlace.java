package com.teispace.teistro;

import java.util.List;

/**
 * Where one of the month's lords stands ({@code 03-design/varshesha.md}, "The month's results").
 *
 * @param office The office.
 * @param graha Who holds it.
 * @param house The house it stands in, 1 to 12 from the month lagna.
 * @param clauses The clauses of vv. 12–13 that hold, in the verses' order.
 */
public record MonthLordPlace(MonthLord office, Graha graha, int house, List<MonthLordClause> clauses) {
}
