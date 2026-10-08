package com.teispace.teistro;

/**
 * The eighth of a sign a transit stands in, 3 degrees 45 minutes each (Phaladeepika ch. 23 v. 16),
 * and its lord in the orbits' order (vv. 18 and 19).
 *
 * @param index Which eighth, 1 to 8.
 * @param lord Its lord.
 */
public record Kakshya(
        int index,
        KakshyaLord lord) {
}
