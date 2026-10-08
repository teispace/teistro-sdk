package com.teispace.teistro;

import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.RecordComponent;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * One named condition from a source, without when it held. Each kind is a
 * record named for its tag ({@link #clause()}), holding that kind's
 * fields; a {@code switch} on the record reads it, and one handed back in
 * a request's {@code bars} bars exactly that clause. A {@code grade} is
 * {@code BEST}, {@code MIDDLING} or {@code REJECTED}.
 *
 * <pre>{@code
 * for (MuhurtaClause clause : window.clauses()) {
 *     if (clause.kind() instanceof MuhurtaClauseKind.TithiClause tithi) {
 *         System.out.println(tithi.tithi() + " " + tithi.grade());
 *     }
 * }
 * }</pre>
 */
public sealed interface MuhurtaClauseKind {
    /** Every kind's tag, in the order the kinds are declared. */
    List<String> CLAUSES = List.of("TITHI", "NAKSHATRA", "YOGA", "KARANA", "VARA", "MONTH", "SOLAR_MONTH", "LAGNA", "PADA", "KAALA", "CHOGHADIYA", "ABHIJIT", "MUHURTA_YOGA", "TARABALA", "CHANDRABALA", "KARTARI", "MOON_IN_DUSTHANA", "MOON_JOINED", "VENUS_IN_SIXTH", "MARS_IN_EIGHTH", "ASHTAMA_LAGNA", "KUNAVAMSA", "PANCHAKA_REMAINDER", "LAGNA_TYAJYA", "SEVENTH_OCCUPIED", "MALEFIC_IN_LAGNA", "BENEFIC_IN_LAGNA", "EXALTED_IN_LAGNA", "LUMINARY_IN_ELEVENTH", "KENDRA_BENEFICS", "UNWANTED_PLACEMENT");

    /**
     * The kind's tag, as the answer and a request spell it.
     *
     * @return the tag
     */
    String clause();

    /**
     * The clause as a request writes it: its tag as {@code clause}, and its
     * fields by their JSON names.
     *
     * @return the request record, for {@link Json#write}
     */
    default Map<String, Object> request() {
        Map<String, Object> out = new LinkedHashMap<>();
        out.put("clause", clause());
        for (RecordComponent component : getClass().getRecordComponents()) {
            try {
                out.put(component.getName(), Reads.written(component.getAccessor().invoke(this)));
            } catch (IllegalAccessException | InvocationTargetException e) {
                throw new IllegalStateException("a clause's field `" + component.getName() + "` cannot be read", e);
            }
        }
        return out;
    }

    /**
     * A clause kind from its tagged JSON.
     *
     * @param raw the JSON object, as {@link Json#read} reads it
     * @return the clause
     */
    static MuhurtaClauseKind read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        String tag = Reads.string(o, "clause");
        return switch (tag) {
            case "TITHI" -> new TithiClause(Reads.member(Tithi.byKey(Reads.string(o, "tithi")), "Tithi", o.get("tithi")), Reads.string(o, "grade"));
            case "NAKSHATRA" -> new NakshatraClause(Reads.member(Nakshatra.byKey(Reads.string(o, "nakshatra")), "Nakshatra", o.get("nakshatra")), Reads.string(o, "grade"));
            case "YOGA" -> new YogaClause(Reads.member(Yoga.byKey(Reads.string(o, "yoga")), "Yoga", o.get("yoga")), Reads.string(o, "grade"));
            case "KARANA" -> new KaranaClause(Reads.member(Karana.byKey(Reads.string(o, "karana")), "Karana", o.get("karana")), Reads.string(o, "grade"));
            case "VARA" -> new VaraClause(Reads.member(Vara.byKey(Reads.string(o, "vara")), "Vara", o.get("vara")), Reads.string(o, "grade"));
            case "MONTH" -> new MonthClause(Reads.member(Masa.byKey(Reads.string(o, "masa")), "Masa", o.get("masa")), Reads.string(o, "grade"));
            case "SOLAR_MONTH" -> new SolarMonthClause(Reads.member(Rashi.byKey(Reads.string(o, "sign")), "Rashi", o.get("sign")), Reads.string(o, "grade"));
            case "LAGNA" -> new LagnaClause(Reads.member(Rashi.byKey(Reads.string(o, "sign")), "Rashi", o.get("sign")), Reads.string(o, "grade"));
            case "PADA" -> new PadaClause(MuhurtaPada.read(Reads.field(o, "pada")));
            case "KAALA" -> new KaalaClause(Reads.member(Kaala.byKey(Reads.string(o, "kaala")), "Kaala", o.get("kaala")));
            case "CHOGHADIYA" -> new ChoghadiyaClause(Reads.member(Choghadiya.byKey(Reads.string(o, "choghadiya")), "Choghadiya", o.get("choghadiya")));
            case "ABHIJIT" -> new AbhijitClause();
            case "MUHURTA_YOGA" -> new MuhurtaYogaClause(Reads.member(MuhurtaYoga.byKey(Reads.string(o, "yoga")), "MuhurtaYoga", o.get("yoga")));
            case "TARABALA" -> new TarabalaClause(TaraReading.read(Reads.field(o, "reading")));
            case "CHANDRABALA" -> new ChandrabalaClause(Reads.integer(o, "house"), Reads.flag(o, "holds"));
            case "KARTARI" -> new KartariClause(grahas(o, "second"), grahas(o, "twelfth"));
            case "MOON_IN_DUSTHANA" -> new MoonInDusthanaClause(Reads.integer(o, "house"));
            case "MOON_JOINED" -> new MoonJoinedClause(grahas(o, "with"));
            case "VENUS_IN_SIXTH" -> new VenusInSixthClause();
            case "MARS_IN_EIGHTH" -> new MarsInEighthClause();
            case "ASHTAMA_LAGNA" -> new AshtamaLagnaClause();
            case "KUNAVAMSA" -> new KunavamsaClause(Reads.member(Rashi.byKey(Reads.string(o, "navamsa")), "Rashi", o.get("navamsa")), Reads.member(Graha.byKey(Reads.string(o, "lord")), "Graha", o.get("lord")));
            case "PANCHAKA_REMAINDER" -> new PanchakaRemainderClause(Reads.member(Panchaka.byKey(Reads.string(o, "panchaka")), "Panchaka", o.get("panchaka")));
            case "LAGNA_TYAJYA" -> new LagnaTyajyaClause(Reads.member(Rashi.byKey(Reads.string(o, "sign")), "Rashi", o.get("sign")));
            case "SEVENTH_OCCUPIED" -> new SeventhOccupiedClause(grahas(o, "by"));
            case "MALEFIC_IN_LAGNA" -> new MaleficInLagnaClause(grahas(o, "grahas"));
            case "BENEFIC_IN_LAGNA" -> new BeneficInLagnaClause(grahas(o, "grahas"));
            case "EXALTED_IN_LAGNA" -> new ExaltedInLagnaClause(grahas(o, "grahas"));
            case "LUMINARY_IN_ELEVENTH" -> new LuminaryInEleventhClause(grahas(o, "grahas"));
            case "KENDRA_BENEFICS" -> new KendraBeneficsClause(grahas(o, "grahas"));
            case "UNWANTED_PLACEMENT" -> new UnwantedPlacementClause(Reads.integer(o, "house"), grahas(o, "by"));
            default -> throw Reads.internal("the library drew a clause this build does not know: " + tag);
        };
    }

    private static List<Graha> grahas(Map<?, ?> o, String key) {
        return Reads.each(o, key, one -> Reads.member(Graha.byKey(Reads.string(one)), "Graha", one));
    }

    /**
     * The day's tithi, as the rules grade it.
     *
     * @param tithi the tithi
     * @param grade {@code BEST}, {@code MIDDLING} or {@code REJECTED}
     */
    record TithiClause(Tithi tithi, String grade) implements MuhurtaClauseKind {
        /**
         * The kind's tag.
         *
         * @return {@code "TITHI"}
         */
        @Override
        public String clause() {
            return "TITHI";
        }
    }

    /**
     * The Moon's nakshatra, as the rules grade it.
     *
     * @param nakshatra the nakshatra
     * @param grade {@code BEST}, {@code MIDDLING} or {@code REJECTED}
     */
    record NakshatraClause(Nakshatra nakshatra, String grade) implements MuhurtaClauseKind {
        /**
         * The kind's tag.
         *
         * @return {@code "NAKSHATRA"}
         */
        @Override
        public String clause() {
            return "NAKSHATRA";
        }
    }

    /**
     * The nitya yoga, as the rules grade it.
     *
     * @param yoga the yoga
     * @param grade {@code BEST}, {@code MIDDLING} or {@code REJECTED}
     */
    record YogaClause(Yoga yoga, String grade) implements MuhurtaClauseKind {
        /**
         * The kind's tag.
         *
         * @return {@code "YOGA"}
         */
        @Override
        public String clause() {
            return "YOGA";
        }
    }

    /**
     * The karana, as the rules grade it.
     *
     * @param karana the karana
     * @param grade {@code BEST}, {@code MIDDLING} or {@code REJECTED}
     */
    record KaranaClause(Karana karana, String grade) implements MuhurtaClauseKind {
        /**
         * The kind's tag.
         *
         * @return {@code "KARANA"}
         */
        @Override
        public String clause() {
            return "KARANA";
        }
    }

    /**
     * The weekday, as the rules grade it.
     *
     * @param vara the vara
     * @param grade {@code BEST}, {@code MIDDLING} or {@code REJECTED}
     */
    record VaraClause(Vara vara, String grade) implements MuhurtaClauseKind {
        /**
         * The kind's tag.
         *
         * @return {@code "VARA"}
         */
        @Override
        public String clause() {
            return "VARA";
        }
    }

    /**
     * The lunar month, as the rules grade it (C161).
     *
     * @param masa the masa
     * @param grade {@code BEST}, {@code MIDDLING} or {@code REJECTED}
     */
    record MonthClause(Masa masa, String grade) implements MuhurtaClauseKind {
        /**
         * The kind's tag.
         *
         * @return {@code "MONTH"}
         */
        @Override
        public String clause() {
            return "MONTH";
        }
    }

    /**
     * The Sun's sign, as the rules grade it (C161).
     *
     * @param sign the sign
     * @param grade {@code BEST}, {@code MIDDLING} or {@code REJECTED}
     */
    record SolarMonthClause(Rashi sign, String grade) implements MuhurtaClauseKind {
        /**
         * The kind's tag.
         *
         * @return {@code "SOLAR_MONTH"}
         */
        @Override
        public String clause() {
            return "SOLAR_MONTH";
        }
    }

    /**
     * The rising sign, as the rules grade it.
     *
     * @param sign the sign
     * @param grade {@code BEST}, {@code MIDDLING} or {@code REJECTED}
     */
    record LagnaClause(Rashi sign, String grade) implements MuhurtaClauseKind {
        /**
         * The kind's tag.
         *
         * @return {@code "LAGNA"}
         */
        @Override
        public String clause() {
            return "LAGNA";
        }
    }

    /**
     * A quarter of the Moon's star the rules reject.
     *
     * @param pada the quarter
     */
    record PadaClause(MuhurtaPada pada) implements MuhurtaClauseKind {
        /**
         * The kind's tag.
         *
         * @return {@code "PADA"}
         */
        @Override
        public String clause() {
            return "PADA";
        }
    }

    /**
     * Rahu kaala, Yamaghanda or Gulika kaala.
     *
     * @param kaala the kaala
     */
    record KaalaClause(Kaala kaala) implements MuhurtaClauseKind {
        /**
         * The kind's tag.
         *
         * @return {@code "KAALA"}
         */
        @Override
        public String clause() {
            return "KAALA";
        }
    }

    /**
     * The choghadiya.
     *
     * @param choghadiya the choghadiya
     */
    record ChoghadiyaClause(Choghadiya choghadiya) implements MuhurtaClauseKind {
        /**
         * The kind's tag.
         *
         * @return {@code "CHOGHADIYA"}
         */
        @Override
        public String clause() {
            return "CHOGHADIYA";
        }
    }

    /**
     * Abhijit muhurta.
     */
    record AbhijitClause() implements MuhurtaClauseKind {
        /**
         * The kind's tag.
         *
         * @return {@code "ABHIJIT"}
         */
        @Override
        public String clause() {
            return "ABHIJIT";
        }
    }

    /**
     * A special yoga of vara, tithi and nakshatra (Raman ch. VI).
     *
     * @param yoga the yoga
     */
    record MuhurtaYogaClause(MuhurtaYoga yoga) implements MuhurtaClauseKind {
        /**
         * The kind's tag.
         *
         * @return {@code "MUHURTA_YOGA"}
         */
        @Override
        public String clause() {
            return "MUHURTA_YOGA";
        }
    }

    /**
     * The native's tarabala.
     *
     * @param reading the tara reading
     */
    record TarabalaClause(TaraReading reading) implements MuhurtaClauseKind {
        /**
         * The kind's tag.
         *
         * @return {@code "TARABALA"}
         */
        @Override
        public String clause() {
            return "TARABALA";
        }
    }

    /**
     * The native's chandrabala: the Moon's house from the birth sign.
     *
     * @param house the house
     * @param holds whether it holds
     */
    record ChandrabalaClause(int house, boolean holds) implements MuhurtaClauseKind {
        /**
         * The kind's tag.
         *
         * @return {@code "CHANDRABALA"}
         */
        @Override
        public String clause() {
            return "CHANDRABALA";
        }
    }

    /**
     * Malefics either side of the lagna.
     *
     * @param second the malefics in the 2nd
     * @param twelfth the malefics in the 12th
     */
    record KartariClause(List<Graha> second, List<Graha> twelfth) implements MuhurtaClauseKind {
        /** Keeps the lists unmodifiable. */
        public KartariClause {
            second = List.copyOf(second);
            twelfth = List.copyOf(twelfth);
        }

        /**
         * The kind's tag.
         *
         * @return {@code "KARTARI"}
         */
        @Override
        public String clause() {
            return "KARTARI";
        }
    }

    /**
     * The Moon in the 6th, 8th or 12th from the lagna.
     *
     * @param house the house
     */
    record MoonInDusthanaClause(int house) implements MuhurtaClauseKind {
        /**
         * The kind's tag.
         *
         * @return {@code "MOON_IN_DUSTHANA"}
         */
        @Override
        public String clause() {
            return "MOON_IN_DUSTHANA";
        }
    }

    /**
     * The Moon with another graha.
     *
     * @param with who (JSON writes {@code with})
     */
    record MoonJoinedClause(List<Graha> with) implements MuhurtaClauseKind {
        /** Keeps the lists unmodifiable. */
        public MoonJoinedClause {
            with = List.copyOf(with);
        }

        /**
         * The kind's tag.
         *
         * @return {@code "MOON_JOINED"}
         */
        @Override
        public String clause() {
            return "MOON_JOINED";
        }
    }

    /**
     * Venus in the 6th (Bhrigu shatka).
     */
    record VenusInSixthClause() implements MuhurtaClauseKind {
        /**
         * The kind's tag.
         *
         * @return {@code "VENUS_IN_SIXTH"}
         */
        @Override
        public String clause() {
            return "VENUS_IN_SIXTH";
        }
    }

    /**
     * Mars in the 8th (Kujashtama).
     */
    record MarsInEighthClause() implements MuhurtaClauseKind {
        /**
         * The kind's tag.
         *
         * @return {@code "MARS_IN_EIGHTH"}
         */
        @Override
        public String clause() {
            return "MARS_IN_EIGHTH";
        }
    }

    /**
     * The lagna eighth from the native's birth lagna.
     */
    record AshtamaLagnaClause() implements MuhurtaClauseKind {
        /**
         * The kind's tag.
         *
         * @return {@code "ASHTAMA_LAGNA"}
         */
        @Override
        public String clause() {
            return "ASHTAMA_LAGNA";
        }
    }

    /**
     * The lagna in a malefic's navamsa.
     *
     * @param navamsa the lagna's navamsa
     * @param lord the navamsa's lord
     */
    record KunavamsaClause(Rashi navamsa, Graha lord) implements MuhurtaClauseKind {
        /**
         * The kind's tag.
         *
         * @return {@code "KUNAVAMSA"}
         */
        @Override
        public String clause() {
            return "KUNAVAMSA";
        }
    }

    /**
     * The panchaka the remainder by nine names (C159).
     *
     * @param panchaka the panchaka
     */
    record PanchakaRemainderClause(Panchaka panchaka) implements MuhurtaClauseKind {
        /**
         * The kind's tag.
         *
         * @return {@code "PANCHAKA_REMAINDER"}
         */
        @Override
        public String clause() {
            return "PANCHAKA_REMAINDER";
        }
    }

    /**
     * The lagna in its rasi visha ghatika.
     *
     * @param sign the sign
     */
    record LagnaTyajyaClause(Rashi sign) implements MuhurtaClauseKind {
        /**
         * The kind's tag.
         *
         * @return {@code "LAGNA_TYAJYA"}
         */
        @Override
        public String clause() {
            return "LAGNA_TYAJYA";
        }
    }

    /**
     * A graha in the 7th.
     *
     * @param by the grahas there
     */
    record SeventhOccupiedClause(List<Graha> by) implements MuhurtaClauseKind {
        /** Keeps the lists unmodifiable. */
        public SeventhOccupiedClause {
            by = List.copyOf(by);
        }

        /**
         * The kind's tag.
         *
         * @return {@code "SEVENTH_OCCUPIED"}
         */
        @Override
        public String clause() {
            return "SEVENTH_OCCUPIED";
        }
    }

    /**
     * A malefic in the lagna.
     *
     * @param grahas the grahas
     */
    record MaleficInLagnaClause(List<Graha> grahas) implements MuhurtaClauseKind {
        /** Keeps the lists unmodifiable. */
        public MaleficInLagnaClause {
            grahas = List.copyOf(grahas);
        }

        /**
         * The kind's tag.
         *
         * @return {@code "MALEFIC_IN_LAGNA"}
         */
        @Override
        public String clause() {
            return "MALEFIC_IN_LAGNA";
        }
    }

    /**
     * Venus, Mercury or Jupiter in the lagna (neutralisation 6).
     *
     * @param grahas the grahas
     */
    record BeneficInLagnaClause(List<Graha> grahas) implements MuhurtaClauseKind {
        /** Keeps the lists unmodifiable. */
        public BeneficInLagnaClause {
            grahas = List.copyOf(grahas);
        }

        /**
         * The kind's tag.
         *
         * @return {@code "BENEFIC_IN_LAGNA"}
         */
        @Override
        public String clause() {
            return "BENEFIC_IN_LAGNA";
        }
    }

    /**
     * An exalted graha in the lagna (neutralisation 10).
     *
     * @param grahas the grahas
     */
    record ExaltedInLagnaClause(List<Graha> grahas) implements MuhurtaClauseKind {
        /** Keeps the lists unmodifiable. */
        public ExaltedInLagnaClause {
            grahas = List.copyOf(grahas);
        }

        /**
         * The kind's tag.
         *
         * @return {@code "EXALTED_IN_LAGNA"}
         */
        @Override
        public String clause() {
            return "EXALTED_IN_LAGNA";
        }
    }

    /**
     * The Sun or the Moon in the 11th (neutralisation 8).
     *
     * @param grahas the grahas
     */
    record LuminaryInEleventhClause(List<Graha> grahas) implements MuhurtaClauseKind {
        /** Keeps the lists unmodifiable. */
        public LuminaryInEleventhClause {
            grahas = List.copyOf(grahas);
        }

        /**
         * The kind's tag.
         *
         * @return {@code "LUMINARY_IN_ELEVENTH"}
         */
        @Override
        public String clause() {
            return "LUMINARY_IN_ELEVENTH";
        }
    }

    /**
     * Jupiter or Venus in a kendra with the Sun, Mars and Saturn in the 3rd, 6th or 11th (neutralisation 11, C167).
     *
     * @param grahas the grahas
     */
    record KendraBeneficsClause(List<Graha> grahas) implements MuhurtaClauseKind {
        /** Keeps the lists unmodifiable. */
        public KendraBeneficsClause {
            grahas = List.copyOf(grahas);
        }

        /**
         * The kind's tag.
         *
         * @return {@code "KENDRA_BENEFICS"}
         */
        @Override
        public String clause() {
            return "KENDRA_BENEFICS";
        }
    }

    /**
     * Grahas standing in a house the rite's rules want them out of, as their {@code unwanted} list names them: one clause a house.
     *
     * @param house the house
     * @param by the grahas there
     */
    record UnwantedPlacementClause(int house, List<Graha> by) implements MuhurtaClauseKind {
        /** Keeps the lists unmodifiable. */
        public UnwantedPlacementClause {
            by = List.copyOf(by);
        }

        /**
         * The kind's tag.
         *
         * @return {@code "UNWANTED_PLACEMENT"}
         */
        @Override
        public String clause() {
            return "UNWANTED_PLACEMENT";
        }
    }
}
