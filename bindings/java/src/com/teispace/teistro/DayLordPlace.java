package com.teispace.teistro;

import java.util.List;

/**
 * Where one of the day's lords stands ({@code 03-design/varshesha.md}, "The day's results").
 *
 * @param office The office.
 * @param graha Who holds it.
 * @param house The house it stands in, 1 to 12 from the day lagna.
 * @param clauses The clauses that hold, the month's lords' own.
 */
public record DayLordPlace(DayLord office, Graha graha, int house, List<MonthLordClause> clauses) {
}
