package com.teispace.teistro;

/**
 * One member of a limb, with its own bounds and the clipped ones.
 *
 * @param member which member ran
 * @param whole when the member itself began and ended, inside the day or not
 * @param inside the part inside the day: what an almanac row prints
 * @param sunrises which of the day's two sunrises the member was running at:
 *     {@code BOTH} when it names two days (vriddhi), {@code NEITHER} when it
 *     names none (kshaya)
 * @param ends when the member ended, in ghati-pala from the day's sunrise
 *     under the day's ghati reckoning; a member outlasting the day reads as
 *     the day's whole count
 * @param marginMinutes how close the member's end came to deciding which day
 *     it names: the minutes from the nearer of the day's two sunrises (its
 *     own and the next) to {@code whole.to}, positive when it ends after that
 *     sunrise and negative before it (SR-95)
 * @param <T> the limb's member type
 */
public record Span<T>(T member, Interval whole, Interval inside, Sunrises sunrises, GhatiPala ends,
        double marginMinutes) {}
