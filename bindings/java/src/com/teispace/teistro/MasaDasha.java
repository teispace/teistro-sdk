package com.teispace.teistro;

import java.util.List;

import com.teispace.teistro.request.MasaDashaRules;

/**
 * A month's dasha ({@code 03-design/varshesha.md}, "The month's dasha"; <i>Tajika
 * Nilakanthi</i>, Varsha Tantra, v. 14): the five Huddas of the stronger lord's sign, each
 * running its degrees in days.
 *
 * @param office Whose lord was the stronger: {@link MonthOffice#MUNTHA} or
 *     {@link MonthOffice#MASA_LAGNA}.
 * @param graha That lord.
 * @param sign The sign whose Huddas run the month.
 * @param rules The readings it was computed under, every one named.
 * @param periods The sign's five Huddas, in order from the month's opening.
 */
public record MasaDasha(MonthOffice office, Graha graha, Rashi sign, MasaDashaRules rules,
        List<MasaDashaPeriod> periods) {
}
