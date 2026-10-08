package com.teispace.teistro;

import java.util.List;

/**
 * A graha a remedy is for, and every reason ({@code FUNCTIONAL_MALEFIC}, {@code DEBILITATED},
 * {@code DUSTHANA}, {@code MARAKA}, {@code MAHADASHA}, {@code ANTARDASHA}, {@code COMBUST} or
 * {@code BADHAKESHA}).
 *
 * @param graha The graha.
 * @param reasons Every reason.
 */
public record RemedySubject(
        Graha graha,
        List<String> reasons) {
    /** The value, its lists copied and unmodifiable. */
    public RemedySubject {
        reasons = List.copyOf(reasons);
    }
}
