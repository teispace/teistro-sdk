package com.teispace.teistro;

import java.util.List;
import java.util.Map;
import java.util.Optional;

/**
 * A sign's period in the reading voice (SR-152, {@code 03-design/rashifal-reading.md}):
 * intents a reading pack words, how firmly each is said, and the facts each
 * rests on, in the order a page shows them.
 *
 * @param rashi the sign
 * @param period the period's kind: {@code DAILY}, {@code WEEKLY}, {@code MONTHLY} or {@code YEARLY}
 * @param headline the line over the reading
 * @param parts the reading, part by part
 * @param why the plain why
 * @param areas the four areas
 * @param bestDay a week's best day as an index into its days, when one day leads; may be null
 * @param lucky the sign's lucky items
 * @param syllables the sign's nine name syllables in IAST, its padas' in order
 */
public record ReadingPlan(Rashi rashi, String period, Item headline, List<Part> parts, List<Item> why,
        List<AreaReading> areas, Integer bestDay, LuckyElements lucky, List<String> syllables) {
    /** Keeps the lists unmodifiable. */
    public ReadingPlan {
        parts = List.copyOf(parts);
        why = List.copyOf(why);
        areas = List.copyOf(areas);
        syllables = List.copyOf(syllables);
    }

    /** How firmly an item is said, from its facts; nothing is certain. */
    public enum Hedge {
        /** An excellent band with two facts agreeing. */
        FIRM,
        /** One good fact. */
        LIKELY,
        /** A good fact obstructed, or a mixed band. */
        POSSIBLE,
        /** A caution. */
        MAY;

        /**
         * The member the wire spells {@code key} with.
         *
         * @param key the wire's spelling
         * @return the member, or empty for one this build does not know
         */
        public static Optional<Hedge> byKey(String key) {
            return keyed(values(), key);
        }
    }

    /** The four areas a reader sees, read from the baseline's life areas. */
    public enum Area {
        /** The baseline's CAREER. */
        WORK,
        /** The baseline's FINANCE. */
        MONEY,
        /** The mean of RELATIONSHIPS and FAMILY (C2359). */
        LOVE_FAMILY,
        /** The baseline's HEALTH. */
        HEALTH;

        /**
         * The member the wire spells {@code key} with.
         *
         * @param key the wire's spelling
         * @return the member, or empty for one this build does not know
         */
        public static Optional<Area> byKey(String key) {
            return keyed(values(), key);
        }
    }

    /** Where in the period a part falls. */
    public enum Frame {
        /** The whole period. */
        WHOLE,
        /** A day before the Moon's change of sign. */
        BEFORE_MOON,
        /** A day after it. */
        AFTER_MOON,
        /** A week's first two days. */
        START,
        /** A week's middle three. */
        MIDDLE,
        /** A week's last two. */
        END,
        /** A month before its largest event. */
        FIRST_HALF,
        /** A month from it. */
        SECOND_HALF,
        /** A year's slow grahas. */
        SLOW,
        /** A year's first quarter. */
        QUARTER1,
        /** Its second. */
        QUARTER2,
        /** Its third. */
        QUARTER3,
        /** Its fourth. */
        QUARTER4;

        /**
         * The member the wire spells {@code key} with.
         *
         * @param key the wire's spelling
         * @return the member, or empty for one this build does not know
         */
        public static Optional<Frame> byKey(String key) {
            return keyed(values(), key);
        }
    }

    private static <E extends Enum<E>> Optional<E> keyed(E[] values, String key) {
        for (E one : values) {
            if (one.name().equals(key)) {
                return Optional.of(one);
            }
        }
        return Optional.empty();
    }

    /**
     * A fact an item rests on, as an index into the answer that carries it.
     *
     * @param kind {@code TRANSIT}, {@code SATURN}, {@code EVENT}, {@code INFLUENCE}, {@code BAND},
     *     {@code AREA_BAND}, {@code DAY}, {@code MOON_TURN}, {@code LUCKY} or {@code SYLLABLES}
     * @param graha a {@code TRANSIT}'s graha; may be null
     * @param index an {@code EVENT}'s, an {@code INFLUENCE}'s or a {@code DAY}'s index; may be null
     * @param area an {@code AREA_BAND}'s area; may be null
     */
    public record Fact(String kind, Graha graha, Integer index, Area area) {}

    /**
     * One thing a reading says; {@code intl().render(item.intent(), item.params())} says it.
     *
     * @param intent the intent, a message key of the reading pack
     * @param params its slots, with its {@code hedge} and its {@code variant}, the seeded number
     *     a pack reads as one of its wordings
     * @param hedge how firmly it is said
     * @param caution whether it is a caution, which the period's budget counts
     * @param restsOn the facts it says; never empty
     */
    public record Item(String intent, Map<String, Object> params, Hedge hedge, boolean caution,
            List<Fact> restsOn) {
        /** Keeps the slots and facts unmodifiable. */
        public Item {
            params = Map.copyOf(params);
            restsOn = List.copyOf(restsOn);
        }
    }

    /**
     * A part of the reading.
     *
     * @param frame where it falls
     * @param items what it says, in order
     */
    public record Part(Frame frame, List<Item> items) {
        /** Keeps the items unmodifiable. */
        public Part {
            items = List.copyOf(items);
        }
    }

    /**
     * One area as a page shows it.
     *
     * @param area the area
     * @param score its score, 0 to 100
     * @param band its band under the overall's thresholds
     * @param note its one-line note
     */
    public record AreaReading(Area area, int score, ScoreBand band, Item note) {}
}
