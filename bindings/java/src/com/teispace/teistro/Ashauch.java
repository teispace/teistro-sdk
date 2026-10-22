package com.teispace.teistro;

import com.teispace.teistro.request.AshauchKind;
import com.teispace.teistro.request.AshauchLength;
import com.teispace.teistro.request.AshauchRelation;
import com.teispace.teistro.request.AshauchRunning;
import com.teispace.teistro.request.AshauchSex;

/**
 * The period of impurity after a birth or a death ({@code 03-design/ashauch.md}), and the clause
 * that set it.
 *
 * @param key the request's key
 * @param kind a birth's or a death's
 * @param relation the request's relation
 * @param sex the deceased's sex, or the child's, as the request gave it; null where it gave none
 * @param start when it began, a UTC Julian day: the event, the cremation, or the hearing
 * @param dayOne the civil day it counts from
 * @param dayOneBy why that day
 * @param end when it ends, a UTC Julian day: a sunrise, a nightfall, or {@code start} itself for a
 *     bath
 * @param lastDay the last day it touches, for a calendar to shade; null for a bath
 * @param length the length its clause named
 * @param days the days, when the length is a count; null otherwise
 * @param clause the clause
 * @param overlap what a period already running did, when it did; null otherwise
 * @param ritesFrom when the mother may take part in rites again, a UTC Julian day, for the mother
 *     of a birth under a table that reports it; null otherwise
 * @param table the table's name
 */
public record Ashauch(
        String key, AshauchKind kind, AshauchRelation relation, AshauchSex sex, double start,
        CalendarDate dayOne, AshauchDayOneBy dayOneBy, double end, CalendarDate lastDay,
        AshauchLength length, Integer days, AshauchClause clause, AshauchOverlapped overlap,
        Double ritesFrom, String table) {
    /**
     * This period as a later request's {@code during} takes it.
     *
     * @return the parts of it the overlap rules read
     */
    public AshauchRunning running() {
        return new AshauchRunning(days, end, key, kind, length, relation, sex, start);
    }
}
