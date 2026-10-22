package com.teispace.teistro;

/**
 * When an Ekadashi fast is broken ({@code 03-design/festival-rules.md}
 * §8.5): the day after the fast, inside the 12th and past its first
 * quarter, the hari vasara.
 *
 * @param day the day after the fast
 * @param tithi the tithi the meal is kept in: the 12th when it stands past
 *     that day's sunrise, and otherwise the tithi at the sunrise
 * @param hariVasaraEnds when the hari vasara ends, a UTC Julian day, if
 *     after that day's sunrise; null otherwise
 * @param kept the window kept; null when none of the rule's leaves time
 * @param barred the rule's nakshatra bar, when one met the parana day;
 *     null otherwise
 */
public record Parana(
        CalendarDate day, Tithi tithi, Double hariVasaraEnds, ParanaKept kept, ParanaBarred barred) {}
