package com.teispace.teistro;

/**
 * One bhava as the houses service reads it.
 *
 * @param number The bhava, 1 to 12.
 * @param sign The sign its <b>middle</b> falls in, which under an unequal division is not the sign
 *     it begins in.
 * @param lord The lord of that sign.
 * @param quadrant Which third of the wheel it stands in.
 */
public record ServiceBhava(
        int number,
        Rashi sign,
        Graha lord,
        Quadrant quadrant) {
}
