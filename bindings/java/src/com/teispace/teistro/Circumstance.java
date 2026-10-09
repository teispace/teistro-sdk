package com.teispace.teistro;

import java.util.List;
import java.util.Optional;

/**
 * The circumstances <i>Brihat Jataka</i> ch. V reads at a candidate instant, and each fact the
 * family gave set against them ({@code 03-design/rectification.md}).
 *
 * @param sky The sky the clauses read.
 * @param father V.1–2.
 * @param presentation V.17.
 * @param lamp V.18.
 * @param attending V.22.
 * @param weights One per fact given, in the order {@code FATHER}, {@code PRESENTATION},
 *     {@code OIL}, {@code WICK}, {@code ATTENDANTS_TOTAL}, {@code ATTENDANTS_INSIDE},
 *     {@code ATTENDANTS_OUTSIDE}; none for a fact absent.
 */
public record Circumstance(
        Sky sky,
        Father father,
        Presentation presentation,
        Lamp lamp,
        Attending attending,
        List<Weight> weights) {
    /** The value, its lists copied and unmodifiable. */
    public Circumstance {
        weights = List.copyOf(weights);
    }

    /**
     * The sky at a candidate instant, as the clauses read it: the lagna, the seven grahas and
     * whether the lagna's lord is retrograde.
     *
     * @param lagnaDeg The lagna, degrees.
     * @param grahasDeg The seven grahas' longitudes, degrees, the Sun to Saturn.
     * @param lordRetrograde Whether the lagna's lord is retrograde, read only by
     *     {@code LAGNA_LORD_MOTION}.
     */
    public record Sky(
            double lagnaDeg,
            List<Double> grahasDeg,
            boolean lordRetrograde) {
        /** The value, its lists copied and unmodifiable. */
        public Sky {
            grahasDeg = List.copyOf(grahasDeg);
        }
    }

    /**
     * V.1–2: whether the father was away, and where.
     *
     * @param moonAspect The Moon's aspect on the lagna's sign (BJ II.13).
     * @param unseen V.1: the Moon does not see the lagna, as the {@code moonSees} rule asks.
     * @param saturnRising V.2: Saturn in the lagna.
     * @param marsSetting V.2: Mars in the 7th.
     * @param moonHemmed V.2: the Moon between Mercury and Venus, one in the 12th from her and the
     *     other in the 2nd, or all three in her sign with her degree between theirs (the 1912
     *     print's note).
     * @param away Whether any of them holds: the father away.
     * @param whereabouts V.1's second half, where V.1 holds and the Sun has fallen from the 10th:
     *     where the father is, {@code ABROAD}, {@code OWN_COUNTRY} or {@code RETURNING}; empty
     *     otherwise.
     * @param sunHouse The Sun's house from the lagna, by sign.
     */
    public record Father(
            Strength moonAspect,
            boolean unseen,
            boolean saturnRising,
            boolean marsSetting,
            boolean moonHemmed,
            boolean away,
            Optional<String> whereabouts,
            int sunHouse) {
    }

    /**
     * V.17's presentation as the sky foretells it.
     *
     * @param by The reading asked: {@code RISING_SIGN} or {@code LAGNA_LORD_MOTION}.
     * @param rising How the rising sign rises.
     * @param lord The lagna's lord.
     * @param lordRetrograde Whether it is retrograde.
     * @param foretold What the reading foretells, {@code HEAD}, {@code FEET} or {@code HANDS}:
     *     under {@code LAGNA_LORD_MOTION} the head for a natural birth and the feet for an
     *     irregular one, where any presentation but the head agrees.
     */
    public record Presentation(
            String by,
            Rising rising,
            Graha lord,
            boolean lordRetrograde,
            String foretold) {
    }

    /**
     * V.18's lamp as the sky foretells it. A level is {@code FULL}, {@code HALF} or
     * {@code SPENT}.
     *
     * @param oil The oil left, one at the start of the Moon's sign and none at its end.
     * @param oilLevel Its nearest level.
     * @param wick The wick left, one at the start of the rising sign and none at its end.
     * @param wickLevel Its nearest level.
     */
    public record Lamp(
            double oil,
            String oilLevel,
            double wick,
            String wickLevel) {
    }

    /**
     * V.22's attendants as the sky foretells them.
     *
     * @param between The grahas between the lagna and the Moon, as the {@code betweenBy} rule
     *     counts.
     * @param visible Those of them in the visible half, the 7th house to the 12th by degree from
     *     the descendant up to the ascendant.
     * @param inside How many inside the room.
     * @param outside How many outside it.
     */
    public record Attending(
            List<Graha> between,
            List<Graha> visible,
            int inside,
            int outside) {
        /** The value, its lists copied and unmodifiable. */
        public Attending {
            between = List.copyOf(between);
            visible = List.copyOf(visible);
        }
    }

    /**
     * One fact given, set against the clause that reads it.
     *
     * @param indication The clause, as {@code FATHER} or {@code OIL}.
     * @param agrees Whether the fact agrees with what the sky foretold.
     */
    public record Weight(
            String indication,
            boolean agrees) {
    }
}
