package com.teispace.teistro;

import java.util.List;

/**
 * One civil day of a native's bird under Pancha Pakshi ({@code 03-design/pakshi.md}). The birds,
 * activities and relations are the system's own words and stay as the library spells them.
 *
 * @param date The civil day.
 * @param reading Its reading; null on a day the Sun does not both rise and set.
 */
public record PakshiDay(
        CalendarDate date,
        Reading reading) {
    /**
     * A native's bird over one day.
     *
     * @param day The day's bounds, weekday and paksha.
     * @param bird The native's bird.
     * @param deathBird The bird dead the whole day and night, beside the yamas (P10).
     * @param deadToday Whether that is the native's bird.
     * @param eaters The birds eating in the first yama of the day and of the night.
     * @param yamas The ten yamas, the day's five and then the night's.
     */
    public record Reading(
            Bounds day,
            String bird,
            String deathBird,
            boolean deadToday,
            List<String> eaters,
            List<Yama> yamas) {
        /** The value, its lists copied and unmodifiable. */
        public Reading {
            eaters = List.copyOf(eaters);
            yamas = List.copyOf(yamas);
        }
    }

    /**
     * A day as Pancha Pakshi reads it.
     *
     * @param sunrise Its sunrise, a Julian day (UTC).
     * @param sunset Its sunset.
     * @param nextSunrise The sunrise that ends it.
     * @param vara The weekday of its sunrise (P9).
     * @param paksha The paksha at its sunrise (P2).
     */
    public record Bounds(
            double sunrise,
            double sunset,
            double nextSunrise,
            Vara vara,
            Paksha paksha) {
    }

    /**
     * One yama.
     *
     * @param half {@code DAY} or {@code NIGHT}.
     * @param yama Which yama of the half, 1 to 5.
     * @param span When.
     * @param activity The native bird's activity.
     * @param quality {@code GOOD}, {@code MIDDLING} or {@code BAD}.
     * @param subs Its sub-periods, in order.
     */
    public record Yama(
            String half,
            int yama,
            Interval span,
            String activity,
            String quality,
            List<Sub> subs) {
        /** The value, its lists copied and unmodifiable. */
        public Yama {
            subs = List.copyOf(subs);
        }
    }

    /**
     * A sub-period.
     *
     * @param activity What is done.
     * @param owner The bird whose main activity it is.
     * @param share Its share of the yama, in 144ths.
     * @param ownerIs How the native regards the owner: {@code FRIEND}, {@code ENEMY}, {@code NEUTRAL}
     *     or {@code OWN} for its own sub-period (P12).
     * @param span When.
     */
    public record Sub(
            String activity,
            String owner,
            int share,
            String ownerIs,
            Interval span) {
    }
}
