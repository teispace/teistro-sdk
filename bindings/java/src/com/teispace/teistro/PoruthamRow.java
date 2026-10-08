package com.teispace.teistro;

/**
 * One consideration: whether it agrees, and what it read.
 *
 * @param agrees Whether it agrees, a lift included.
 * @param lifted Whether it agrees only by the p. 76 exception.
 * @param reading What it read.
 */
public record PoruthamRow(
        boolean agrees,
        boolean lifted,
        PoruthamReading reading) {
}
