package com.teispace.teistro;

/**
 * The lunar month a day falls in, under both conventions.
 *
 * @param month the month under the profile's own convention
 * @param amanta the amanta month: new moon to new moon
 * @param purnimanta the purnimanta month: full moon to full moon
 * @param paksha which fortnight the day opens in
 * @param convention which convention {@code month} leads with
 * @param kind whether the month is ordinary, intercalary or omitted. The
 *     name needs no case for the intercalary one (an adhika month and the
 *     nija month after it take the same name), so this is the mark beside
 *     the name
 */
public record Month(Masa month, Masa amanta, Masa purnimanta, Paksha paksha, LunarMonth convention, MonthKind kind) {}
