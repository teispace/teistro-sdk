package com.teispace.teistro;

/**
 * What the Tajika links say of the matter (Prashna Tantra vv. 14 to 21).
 *
 * @param kind {@code DIRECT}, {@code THROUGH}, {@code DESTROYED}, {@code SEPARATED},
 *     {@code SAME_LORD} or {@code UNLINKED}.
 * @param meaning What it says under the question's tense: {@code PROMISED}, {@code FAILS},
 *     {@code PAST}, {@code FAILS_OR_PAST} or {@code UNREAD}.
 * @param yoga The yoga that made it: Ithasala, Nakta, Yamaya, Manau or Ishrafa. May be null.
 * @param through The third graha carrying or gathering the light, or the malefic. May be null.
 * @param kamboola The Kamboola's grade, when the Moon joins the Ithasala. May be null.
 * @param moonless v. 96: whether the Moon looks at neither lord; null where the rules'
 *     {@code moonScope} does not reach the question. May be null.
 * @param samarasimha Samarasimha's conditions that hold, in his order.
 */
public record PrashnaLink(
        String kind,
        String meaning,
        YearYoga yoga,
        Graha through,
        KamboolaGrade kamboola,
        Boolean moonless,
        java.util.List<String> samarasimha) {
    /** The value, its lists copied and unmodifiable. */
    public PrashnaLink {
        samarasimha = java.util.List.copyOf(samarasimha);
    }
}
