package com.teispace.teistro;

import com.teispace.teistro.request.AshauchRite;
import com.teispace.teistro.request.RiteBeginning;
import com.teispace.teistro.request.RiteVerdict;

/**
 * What {@link AlmanacArea#ashauchRite} answers ({@code 03-design/ashauch.md} §12).
 *
 * @param rite the rite, as asked
 * @param verdict whether it goes on, in part, or waits
 * @param beginning what opens the rite, when the texts name it; null otherwise
 * @param row the row's index in the table
 * @param clause the text and page
 * @param table the table's name
 */
public record AshauchRiteAnswer(AshauchRite rite, RiteVerdict verdict, RiteBeginning beginning, int row,
        String clause, String table) {}
