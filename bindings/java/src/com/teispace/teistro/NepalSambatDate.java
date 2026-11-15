package com.teispace.teistro;

/**
 * A day's Nepal Sambat date, the committee's "ने.सं. ११४६ (कछलाथ्व)" with
 * the day of the half Newar dates add, "कछलाथ्व १"
 * ({@code 03-design/calendar-indian-lunisolar.md} §11).
 * {@code sdk.calendar.nepalSambatDate} says one.
 *
 * @param year the year, which opens at Kachhala's first day: 1146 from 2025-10-22
 * @param month the month, 1 for Kachhala (amanta Kartika) to 12 for Kaula
 *     (amanta Ashwina); an adhika month keeps the number of the month it repeats
 * @param kind whether the month is ordinary, intercalary (Anala) or omitted
 * @param paksha the half: Shukla is thwa and Krishna ga
 * @param day the day of the half, the number of the tithi at the day's sunrise within its half, 1 to 15: thwa 15 is the full moon (punhi), ga 15 the new moon (amai)
 */
public record NepalSambatDate(int year, int month, MonthKind kind, Paksha paksha, int day) {}
