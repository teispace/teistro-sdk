package com.teispace.teistro;

import java.util.ArrayList;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import com.teispace.teistro.ffi.Native;

/**
 * What a chart is founded with beyond its instant and place: the sections
 * to compute, the divisional charts, dashas and drawings, and each of the
 * readings a request object names.
 *
 * <pre>{@code
 * ChartOptions options = ChartOptions.builder()
 *         .state(true)
 *         .vargas(Varga.D9)
 *         .dashas(DashaSystem.VIMSHOTTARI)
 *         .kp(Map.of("number", 108))
 *         .build();
 * }</pre>
 *
 * <p>A reading's request object is written as JSON as {@link Json#write}
 * writes it, so a catalogue member may stand for its key; the library
 * checks each and refuses a mistake naming the field.
 *
 * @param kind what kind of chart to found
 * @param sections the sections beside the foundation, as the boundary's bit set
 * @param vargas the divisional charts, in the order to answer them
 * @param dashas the dasha systems: a {@link DashaSystem}, or the full key of one the context registered
 * @param drawings the drawings, each a layout and a varga
 * @param readings each reading's request object as JSON, by the boundary's field name
 */
public record ChartOptions(
        ChartKind kind,
        long sections,
        List<Varga> vargas,
        List<Object> dashas,
        List<Drawing> drawings,
        Map<String, String> readings) {

    /** The options, copied, with nothing null. */
    public ChartOptions {
        kind = kind == null ? ChartKind.NATAL : kind;
        vargas = List.copyOf(vargas);
        dashas = List.copyOf(dashas);
        drawings = List.copyOf(drawings);
        readings = Collections.unmodifiableMap(new LinkedHashMap<>(readings));
    }

    /**
     * A drawing to make: a layout, a {@link ChartLayout} or the full key of one
     * the context registered, of a divisional chart.
     *
     * @param layout the layout
     * @param varga the chart to draw, {@link Varga#D1} for the founded one
     */
    public record Drawing(Object layout, Varga varga) {}

    /**
     * Options with nothing asked for beyond the foundation.
     *
     * @return the options
     */
    public static ChartOptions none() {
        return builder().build();
    }

    /**
     * A builder over nothing asked for.
     *
     * @return the builder
     */
    public static Builder builder() {
        return new Builder();
    }

    /** Builds {@link ChartOptions}, one choice at a time. */
    public static final class Builder {
        private ChartKind kind = ChartKind.NATAL;
        private long sections;
        private final List<Varga> vargas = new ArrayList<>();
        private final List<Object> dashas = new ArrayList<>();
        private final List<Drawing> drawings = new ArrayList<>();
        private final Map<String, String> readings = new LinkedHashMap<>();

        private Builder() {}

        private Builder section(long bit, boolean on) {
            sections = on ? sections | bit : sections & ~bit;
            return this;
        }

        private Builder json(String field, Object request) {
            if (request == null) {
                readings.remove(field);
            } else {
                readings.put(field, request instanceof String json ? json : Json.write(request));
            }
            return this;
        }

        /**
         * What kind of chart to found.
         *
         * @param kind the kind
         * @return this builder
         */
        public Builder kind(ChartKind kind) {
            this.kind = kind;
            return this;
        }

        /**
         * The planetary states: dignity, friendship, combustion, avasthas.
         *
         * @param on whether to compute them
         * @return this builder
         */
        public Builder state(boolean on) {
            return section(Native.TS_CHART_STATE, on);
        }

        /**
         * The drishti between grahas.
         *
         * @param on whether to compute them
         * @return this builder
         */
        public Builder aspects(boolean on) {
            return section(Native.TS_CHART_ASPECTS, on);
        }

        /**
         * The derived points: upagrahas, special lagnas, sahams.
         *
         * @param on whether to compute them
         * @return this builder
         */
        public Builder points(boolean on) {
            return section(Native.TS_CHART_POINTS, on);
        }

        /**
         * The bhavas and their lords.
         *
         * @param on whether to compute them
         * @return this builder
         */
        public Builder houses(boolean on) {
            return section(Native.TS_CHART_HOUSES, on);
        }

        /**
         * The Ashtakavarga.
         *
         * @param on whether to compute it
         * @return this builder
         */
        public Builder ashtakavarga(boolean on) {
            return section(Native.TS_CHART_ASHTAKAVARGA, on);
        }

        /**
         * The Vimshopaka balas.
         *
         * @param on whether to compute them
         * @return this builder
         */
        public Builder vimshopaka(boolean on) {
            return section(Native.TS_CHART_VIMSHOPAKA, on);
        }

        /**
         * The Vaiseshikamsas.
         *
         * @param on whether to compute them
         * @return this builder
         */
        public Builder vaiseshikamsa(boolean on) {
            return section(Native.TS_CHART_VAISESHIKAMSA, on);
        }

        /**
         * Each dasha lord's fruition.
         *
         * @param on whether to compute it
         * @return this builder
         */
        public Builder dashaPhala(boolean on) {
            return section(Native.TS_CHART_DASHA_PHALA, on);
        }

        /**
         * The Jaimini reading: chara karakas, karakamsha, Brahma.
         *
         * @param on whether to compute it
         * @return this builder
         */
        public Builder jaimini(boolean on) {
            return section(Native.TS_CHART_JAIMINI, on);
        }

        /**
         * The Avakahada chakra.
         *
         * @param on whether to compute it
         * @return this builder
         */
        public Builder avakahada(boolean on) {
            return section(Native.TS_CHART_AVAKAHADA, on);
        }

        /**
         * Uranus, Neptune and Pluto beside the nine.
         *
         * @param on whether to place them
         * @return this builder
         */
        public Builder outerPlanets(boolean on) {
            return section(Native.TS_CHART_OUTER, on);
        }

        /**
         * The Shadbala.
         *
         * @param on whether to compute it
         * @return this builder
         */
        public Builder shadbala(boolean on) {
            return section(Native.TS_CHART_SHADBALA, on);
        }

        /**
         * The Bhava bala.
         *
         * @param on whether to compute it
         * @return this builder
         */
        public Builder bhavaBala(boolean on) {
            return section(Native.TS_CHART_BHAVA_BALA, on);
        }

        /**
         * The divisional charts, in the order to answer them.
         *
         * @param vargas the vargas
         * @return this builder
         */
        public Builder vargas(Varga... vargas) {
            this.vargas.addAll(List.of(vargas));
            return this;
        }

        /**
         * The dasha systems, their periods to the settings' depth.
         *
         * @param systems each a {@link DashaSystem}, or the full key of a system the context registered
         * @return this builder
         */
        public Builder dashas(Object... systems) {
            this.dashas.addAll(List.of(systems));
            return this;
        }

        /**
         * A drawing.
         *
         * @param layout a {@link ChartLayout}, or the full key of a layout the context registered
         * @param varga the chart to draw
         * @return this builder
         */
        public Builder drawing(Object layout, Varga varga) {
            this.drawings.add(new Drawing(layout, varga));
            return this;
        }

        /**
         * The theme every drawing is written as SVG in.
         *
         * @param theme an object of {@code style} and {@code content}, or its JSON
         * @return this builder
         */
        public Builder theme(Object theme) {
            return json("theme_json", theme);
        }

        /**
         * Rules to answer over every chart.
         *
         * @param rules the rule request, or its JSON
         * @return this builder
         */
        public Builder rules(Object rules) {
            return json("rules_json", rules);
        }

        /**
         * Narrative plans to compose over every chart.
         *
         * @param interpret the plan request, or its JSON
         * @return this builder
         */
        public Builder interpret(Object interpret) {
            return json("interpret_json", interpret);
        }

        /**
         * A reading the boundary takes as a JSON request object, by its field's
         * name less {@code _json} ({@code kp}, {@code western_aspects} …): the
         * way to every reading, a newer library's among them, each also under
         * its own name below.
         *
         * @param field the field of {@code ts_chart_request}, less {@code _json}
         * @param request the request object, or its JSON; null takes it away
         * @return this builder
         */
        public Builder reading(String field, Object request) {
            return json(field + "_json", request);
        }

        /**
         * The varsha.
         *
         * @param request the request object, or its JSON
         * @return this builder
         */
        public Builder varsha(Object request) {
            return json("varsha_json", request);
        }

        /**
         * The gochar.
         *
         * @param request the request object, or its JSON
         * @return this builder
         */
        public Builder gochar(Object request) {
            return json("gochar_json", request);
        }

        /**
         * The transit hit.
         *
         * @param request the request object, or its JSON
         * @return this builder
         */
        public Builder hits(Object request) {
            return json("hits_json", request);
        }

        /**
         * The Sade Sati.
         *
         * @param request the request object, or its JSON
         * @return this builder
         */
        public Builder sadeSati(Object request) {
            return json("sade_sati_json", request);
        }

        /**
         * The KP.
         *
         * @param request the request object, or its JSON
         * @return this builder
         */
        public Builder kp(Object request) {
            return json("kp_json", request);
        }

        /**
         * The dignities.
         *
         * @param request the request object, or its JSON
         * @return this builder
         */
        public Builder dignities(Object request) {
            return json("dignities_json", request);
        }

        /**
         * The fortitudes.
         *
         * @param request the request object, or its JSON
         * @return this builder
         */
        public Builder fortitudes(Object request) {
            return json("fortitudes_json", request);
        }

        /**
         * The lots.
         *
         * @param request the request object, or its JSON
         * @return this builder
         */
        public Builder lots(Object request) {
            return json("lots_json", request);
        }

        /**
         * The considerations.
         *
         * @param request the request object, or its JSON
         * @return this builder
         */
        public Builder considerations(Object request) {
            return json("considerations_json", request);
        }

        /**
         * The perfection.
         *
         * @param request the request object, or its JSON
         * @return this builder
         */
        public Builder perfection(Object request) {
            return json("perfection_json", request);
        }

        /**
         * The progressions.
         *
         * @param request the request object, or its JSON
         * @return this builder
         */
        public Builder progressions(Object request) {
            return json("progressions_json", request);
        }

        /**
         * The Western aspects.
         *
         * @param request the request object, or its JSON
         * @return this builder
         */
        public Builder westernAspects(Object request) {
            return json("western_aspects_json", request);
        }

        /**
         * The synastry.
         *
         * @param request the request object, or its JSON
         * @return this builder
         */
        public Builder synastry(Object request) {
            return json("synastry_json", request);
        }

        /**
         * The parallels.
         *
         * @param request the request object, or its JSON
         * @return this builder
         */
        public Builder parallels(Object request) {
            return json("parallels_json", request);
        }

        /**
         * The antiscia.
         *
         * @param request the request object, or its JSON
         * @return this builder
         */
        public Builder antiscia(Object request) {
            return json("antiscia_json", request);
        }

        /**
         * The midpoints.
         *
         * @param request the request object, or its JSON
         * @return this builder
         */
        public Builder midpoints(Object request) {
            return json("midpoints_json", request);
        }

        /**
         * The Western houses.
         *
         * @param request the request object, or its JSON
         * @return this builder
         */
        public Builder westernHouses(Object request) {
            return json("western_houses_json", request);
        }

        /**
         * The harmonic.
         *
         * @param request the request object, or its JSON
         * @return this builder
         */
        public Builder harmonic(Object request) {
            return json("harmonic_json", request);
        }

        /**
         * The matching.
         *
         * @param request the request object, or its JSON
         * @return this builder
         */
        public Builder matching(Object request) {
            return json("matching_json", request);
        }

        /**
         * The prashna.
         *
         * @param request the request object, or its JSON
         * @return this builder
         */
        public Builder prashna(Object request) {
            return json("prashna_json", request);
        }

        /**
         * The remedies.
         *
         * @param request the request object, or its JSON
         * @return this builder
         */
        public Builder remedies(Object request) {
            return json("remedies_json", request);
        }

        /**
         * Lal Kitab, the 1952 edition: every member optional, as
         * {@code Map.of("cycle", Map.of("planet", Graha.VENUS, "year", 17), "year", 30)}
         * ({@code 03-design/lalkitab.md}).
         *
         * @param request the request object, or its JSON
         * @return this builder
         */
        public Builder lalkitab(Object request) {
            return json("lalkitab_json", request);
        }

        /**
         * The rectification: each reading asked by its member, as
         * {@code Map.of("purify", Map.of("minutes", 30))}.
         *
         * @param request the request object, or its JSON
         * @return this builder
         */
        public Builder rectification(Object request) {
            return json("rectification_json", request);
        }

        /**
         * The options.
         *
         * @return the options
         */
        public ChartOptions build() {
            return new ChartOptions(kind, sections, vargas, dashas, drawings, readings);
        }
    }
}
