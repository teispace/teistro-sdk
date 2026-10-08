package com.teispace.teistro;

/**
 * A balance written as a reader writes it.
 *
 * @param years Whole years of the year length.
 * @param months Whole months of a twelfth of it.
 * @param days Whole days.
 * @param hours Hours.
 * @param minutes Minutes, rounded.
 */
public record WrittenBalance(
        long years,
        int months,
        int days,
        int hours,
        int minutes) {
}
