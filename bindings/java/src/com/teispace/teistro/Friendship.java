package com.teispace.teistro;

/**
 * How a body stands to its dispositor, three ways.
 *
 * @param natural The table's own reading.
 * @param temporary Where the dispositor stands.
 * @param compound The five-fold compound of the two.
 * @param dispositor The lord of the sign, which all three are with; null only for a body the
 *     catalogue gives no sign. May be null.
 */
public record Friendship(
        Relationship natural,
        Relationship temporary,
        Relationship compound,
        Graha dispositor) {
}
