package com.teispace.teistro;

import java.util.List;

/**
 * The Sarvatobhadra chakra at one instant against the native ({@code 03-design/gochar.md} §8):
 * clauses, scored by nothing. The chakra's own words ({@code FAST}, {@code LEFT}, {@code MALEFIC},
 * {@code NAKSHATRA} and the rest) stay as the library spells them.
 *
 * @param rules The readings it was taken under: {@code front}, {@code sight} and {@code corner}.
 * @param nativePoints The native's points.
 * @param grahas Each graha's, the Sun to Ketu.
 * @param points Each of the native's points with the malefics and benefics that strike it.
 * @param maleficPoints How many of v. 28's five (star, letter, vowel, tithi, sign) a malefic strikes.
 */
public record Sarvatobhadra(
        Rules rules,
        Native nativePoints,
        List<GrahaVedha> grahas,
        List<PointVedha> points,
        int maleficPoints) {
    /** The value, its lists copied and unmodifiable. */
    public Sarvatobhadra {
        grahas = List.copyOf(grahas);
        points = List.copyOf(points);
    }

    /**
     * The readings the texts leave open (C429 to C431).
     *
     * @param front What the front strikes: {@code STAR_ONLY} or {@code EVERY_CELL}.
     * @param sight Which lines a graha looks along: {@code TEXT}, {@code ALL_THREE} or {@code ONE_EACH}.
     * @param corner What a corner's juncture strikes: {@code FOUR_VOWELS} or {@code OUTER_VOWEL}.
     */
    public record Rules(
            String front,
            String sight,
            String corner) {
    }

    /**
     * The native's points.
     *
     * @param nakshatra The natal Moon's star.
     * @param rashi The natal Moon's sign.
     * @param tithi The birth's tithi.
     * @param vara The birth's weekday.
     * @param name The name's sounds; null when none was given.
     */
    public record Native(
            Nakshatra nakshatra,
            Rashi rashi,
            Tithi tithi,
            Vara vara,
            Name name) {
    }

    /**
     * A name's sounds as the chakra reads them.
     *
     * @param letter The first letter in Devanagari.
     * @param vowel The vowel of its first syllable, {@code A} to {@code AH}.
     */
    public record Name(
            String letter,
            String vowel) {
    }

    /**
     * A star of the chakra's 28.
     *
     * @param nakshatra The nakshatra; null for Abhijit.
     */
    public record Star(
            Nakshatra nakshatra) {
        /**
         * Whether it is Abhijit, between Uttarashadha and Shravana.
         *
         * @return whether it is Abhijit
         */
        public boolean abhijit() {
            return nakshatra == null;
        }
    }

    /**
     * One cell of the chakra; only its {@code kind}'s member is set.
     *
     * @param kind {@code STAR}, {@code VOWEL}, {@code LETTER}, {@code RASHI} or {@code TITHI}.
     * @param star The star, for a star's cell.
     * @param vowel The vowel, for a vowel's cell.
     * @param letter The letter in Devanagari, for a letter's cell.
     * @param rashi The sign, for a sign's cell.
     * @param tithiClass The tithi group, for a tithi's cell.
     */
    public record Cell(
            String kind,
            Star star,
            String vowel,
            String letter,
            Rashi rashi,
            TithiClass tithiClass) {
    }

    /**
     * How a graha reaches one thing it strikes.
     *
     * @param by {@code LINE}, {@code CORNER} or {@code STANDS} (in the star whose pada a letter is).
     * @param line The line, for {@code LINE}; otherwise null.
     * @param cell The cell struck; null for {@code STANDS}.
     */
    public record Reach(
            String by,
            String line,
            Cell cell) {
    }

    /**
     * Where a graha stands.
     *
     * @param star The chakra's star.
     * @param nakshatra The nakshatra of the 27.
     * @param pada Its pada, 1 to 4.
     */
    public record Place(
            Star star,
            Nakshatra nakshatra,
            int pada) {
    }

    /**
     * A native's point one graha strikes.
     *
     * @param point The point.
     * @param by Every way it strikes it.
     */
    public record PointStrike(
            String point,
            List<Reach> by) {
        /** The value, its list copied and unmodifiable. */
        public PointStrike {
            by = List.copyOf(by);
        }
    }

    /**
     * What one transiting graha strikes.
     *
     * @param graha The graha.
     * @param place Where it stands.
     * @param motion {@code FAST}, {@code MEAN} or {@code RETROGRADE}.
     * @param lines The lines it looks along: {@code LEFT}, {@code FRONT}, {@code RIGHT}.
     * @param nature {@code BENEFIC} or {@code MALEFIC}.
     * @param waning For the Moon, whether it wanes, which makes it malefic (v. 12).
     * @param joinedBy For a benefic, the malefics in its sign, which make it malefic.
     * @param cells Every cell it strikes, and how.
     * @param points The native's points among them.
     */
    public record GrahaVedha(
            Graha graha,
            Place place,
            String motion,
            List<String> lines,
            String nature,
            boolean waning,
            List<Graha> joinedBy,
            List<Reach> cells,
            List<PointStrike> points) {
        /** The value, its lists copied and unmodifiable. */
        public GrahaVedha {
            lines = List.copyOf(lines);
            joinedBy = List.copyOf(joinedBy);
            cells = List.copyOf(cells);
            points = List.copyOf(points);
        }
    }

    /**
     * Who strikes one of the native's points.
     *
     * @param point {@code NAKSHATRA}, {@code LETTER}, {@code VOWEL}, {@code TITHI}, {@code RASHI} or {@code VARA}.
     * @param malefics The malefics that strike it, in the catalogue's order.
     * @param benefics The benefics that strike it.
     */
    public record PointVedha(
            String point,
            List<Graha> malefics,
            List<Graha> benefics) {
        /** The value, its lists copied and unmodifiable. */
        public PointVedha {
            malefics = List.copyOf(malefics);
            benefics = List.copyOf(benefics);
        }
    }
}
