package com.teispace.teistro;

import java.util.List;

/**
 * A chart read as Lal Kitab reads it, the 1952 edition ({@code 03-design/lalkitab.md}). The book's
 * own words ({@code PAKKA}, {@code FRIEND}, {@code QUARTER}, {@code PITRI} and the rest) stay as the
 * library spells them.
 *
 * @param reading What the teva says.
 * @param cycle Where the 35-year cycle was started.
 * @param periods The cycle's periods over years 1 to 120 of life.
 * @param year The year asked for; null unless the request named one.
 */
public record LalKitab(
        Reading reading,
        Cycle cycle,
        List<Period> periods,
        Year year) {
    /** The value, its lists copied and unmodifiable. */
    public LalKitab {
        periods = List.copyOf(periods);
    }

    /**
     * Where the 35-year cycle starts.
     *
     * @param planet The planet that rules {@code year}.
     * @param year The year of life it rules, from 1.
     */
    public record Cycle(
            Graha planet,
            int year) {
    }

    /**
     * One period of the cycle, both years of life included.
     *
     * @param planet The planet that rules it.
     * @param from Its first year.
     * @param to Its last year.
     */
    public record Period(
            Graha planet,
            int from,
            int to) {
    }

    /**
     * The year of life asked for.
     *
     * @param year The year, from 1.
     * @param ruler The cycle's planet that rules it.
     * @param thirds The planets of months 1–4, 5–8 and 9–12 (1952 p. 34).
     * @param annual The annual teva's reading; null without a {@code varshphal} list.
     */
    public record Year(
            int year,
            Graha ruler,
            List<Graha> thirds,
            Reading annual) {
        /** The value, its lists copied and unmodifiable. */
        public Year {
            thirds = List.copyOf(thirds);
        }
    }

    /**
     * What a teva says ({@code 03-design/lalkitab.md} §3).
     *
     * @param planets Each planet in the teva.
     * @param houses Each of the twelve houses.
     * @param masnui The artificial planets pairs in one house make (1952 p. 27).
     * @param rinas The debts the teva carries (1952 p. 125).
     * @param pitri The ancestors' debt's first state (1952 p. 128).
     * @param flags The teva's named conditions.
     */
    public record Reading(
            List<Planet> planets,
            List<House> houses,
            List<Pair> masnui,
            List<Debt> rinas,
            List<Pitri> pitri,
            Flags flags) {
        /** The value, its lists copied and unmodifiable. */
        public Reading {
            planets = List.copyOf(planets);
            houses = List.copyOf(houses);
            masnui = List.copyOf(masnui);
            rinas = List.copyOf(rinas);
            pitri = List.copyOf(pitri);
        }
    }

    /**
     * One planet in the teva.
     *
     * @param graha The planet.
     * @param house Its house, 1 to 12, the whole-sign house from the lagna.
     * @param dignities Each of {@code PAKKA}, {@code EXALTED}, {@code DEBILITATED} and {@code OWN} it
     *     holds.
     * @param owners The house's owners and how this planet regards each (1952 p. 31).
     * @param awake Whether it is awake.
     * @param kayam In a dignity, alone in its house and looked at from no occupied house.
     * @param casts The aspects it casts, forward only.
     */
    public record Planet(
            Graha graha,
            int house,
            List<String> dignities,
            List<Owner> owners,
            boolean awake,
            boolean kayam,
            List<Cast> casts) {
        /** The value, its lists copied and unmodifiable. */
        public Planet {
            dignities = List.copyOf(dignities);
            owners = List.copyOf(owners);
            casts = List.copyOf(casts);
        }
    }

    /**
     * An owner of a planet's house.
     *
     * @param owner The owner.
     * @param regard {@code FRIEND}, {@code EQUAL} or {@code ENEMY}: how the planet regards it.
     */
    public record Owner(
            Graha owner,
            String regard) {
    }

    /**
     * An aspect a planet casts.
     *
     * @param to The house it looks at.
     * @param strength {@code QUARTER}, {@code HALF} or {@code FULL}.
     * @param onto The planets in that house.
     */
    public record Cast(
            int to,
            String strength,
            List<Graha> onto) {
        /** The value, its lists copied and unmodifiable. */
        public Cast {
            onto = List.copyOf(onto);
        }
    }

    /**
     * One house of the teva.
     *
     * @param house The house, 1 to 12.
     * @param occupants The planets in it.
     * @param lookedAtBy The occupied houses that look at it.
     * @param awake Occupied, or looked at from an occupied house.
     * @param waker The planet whose presence wakes it (1952 p. 98).
     */
    public record House(
            int house,
            List<Graha> occupants,
            List<Look> lookedAtBy,
            boolean awake,
            Graha waker) {
        /** The value, its lists copied and unmodifiable. */
        public House {
            occupants = List.copyOf(occupants);
            lookedAtBy = List.copyOf(lookedAtBy);
        }
    }

    /**
     * An occupied house that looks at another.
     *
     * @param from The house that looks.
     * @param strength {@code QUARTER}, {@code HALF} or {@code FULL}.
     */
    public record Look(
            int from,
            String strength) {
    }

    /**
     * A pair in one house and the artificial planet it makes.
     *
     * @param pair The two planets.
     * @param house Their house.
     * @param countsAs The artificial planet, as {@code MARS_BENEFIC} or {@code SATURN_LIKE_KETU}.
     */
    public record Pair(
            List<Graha> pair,
            int house,
            String countsAs) {
        /** The value, its lists copied and unmodifiable. */
        public Pair {
            pair = List.copyOf(pair);
        }
    }

    /**
     * A debt the teva carries.
     *
     * @param rin The debt, as {@code PITRI} or {@code MATRI}.
     * @param of The planet whose houses its enemies sit in.
     * @param seated Each enemy seated there.
     */
    public record Debt(
            String rin,
            Graha of,
            List<Seat> seated) {
        /** The value, its lists copied and unmodifiable. */
        public Debt {
            seated = List.copyOf(seated);
        }
    }

    /**
     * An enemy seated in one of the indebted planet's houses.
     *
     * @param enemy The enemy.
     * @param house Its house.
     */
    public record Seat(
            Graha enemy,
            int house) {
    }

    /**
     * The ancestors' debt's first state: a planet in 9 with Mercury in its root.
     *
     * @param ninth The planet in the ninth.
     * @param mercury Mercury's house.
     */
    public record Pitri(
            Graha ninth,
            int mercury) {
    }

    /**
     * The teva's named conditions.
     *
     * @param ratandha Whether the teva is ratandha.
     * @param nabalig Whether the teva is nabalig.
     * @param dharmi The dharmi planets.
     * @param sathi Each pair of companion planets.
     */
    public record Flags(
            boolean ratandha,
            boolean nabalig,
            List<Graha> dharmi,
            List<List<Graha>> sathi) {
        /** The value, its lists copied and unmodifiable. */
        public Flags {
            dharmi = List.copyOf(dharmi);
            sathi = sathi.stream().map(List::copyOf).toList();
        }
    }
}
