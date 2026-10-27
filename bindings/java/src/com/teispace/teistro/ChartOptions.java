package com.teispace.teistro;

import java.util.ArrayList;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import com.teispace.teistro.request.AntisciaRequest;
import com.teispace.teistro.request.WesternAspectRequest;
import com.teispace.teistro.request.ConfigurationRequest;
import com.teispace.teistro.request.DignityRequest;
import com.teispace.teistro.request.FortitudeRequest;
import com.teispace.teistro.request.GocharRequest;
import com.teispace.teistro.request.HarmonicRequest;
import com.teispace.teistro.request.HitRequest;
import com.teispace.teistro.request.HouseRequest;
import com.teispace.teistro.request.KpRequest;
import com.teispace.teistro.request.LalKitabRequest;
import com.teispace.teistro.request.SarvatobhadraRequest;
import com.teispace.teistro.request.DossierRequest;
import com.teispace.teistro.request.MasaPraveshaRequest;
import com.teispace.teistro.request.TithiPraveshaRequest;
import com.teispace.teistro.request.LotRequest;
import com.teispace.teistro.request.MidpointRequest;
import com.teispace.teistro.request.ParallelRequest;
import com.teispace.teistro.request.MatchingRequest;
import com.teispace.teistro.request.PartnerSynastry;
import com.teispace.teistro.request.PerfectionRequest;
import com.teispace.teistro.request.PlanRequest;
import com.teispace.teistro.request.PrashnaRequest;
import com.teispace.teistro.request.ProgressionsRequest;
import com.teispace.teistro.request.RectificationRequest;
import com.teispace.teistro.request.RemedyRequest;
import com.teispace.teistro.request.RuleRequest;
import com.teispace.teistro.request.SadeSatiRequest;
import com.teispace.teistro.request.ThemePatch;
import com.teispace.teistro.request.VarshaRequest;

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
 *         .kp(KpRequest.builder().number(108).build())
 *         .build();
 * }</pre>
 *
 * <p>Each reading takes its request type from
 * {@code com.teispace.teistro.request}, which checks a required field, a
 * width and a member's key as it is made, so a misspelt field cannot be
 * written. {@link Builder#reading(String, Object)} takes any reading by
 * its field's name as a map or its JSON, a newer library's among them.
 *
 * @param kind what kind of chart to found
 * @param lilith the lunar apogee to place as the Black Moon Lilith, {@link Lilith#NONE} for none
 * @param sections the sections beside the foundation, as the boundary's bit set
 * @param vargas the divisional charts, in the order to answer them
 * @param dashas the dasha systems: a {@link DashaSystem}, or the full key of one the context registered
 * @param drawings the drawings, each a layout and a varga
 * @param readings each reading's request object as JSON, by the boundary's field name
 */
public record ChartOptions(
        ChartKind kind,
        Lilith lilith,
        long sections,
        List<Varga> vargas,
        List<Object> dashas,
        List<Drawing> drawings,
        Map<String, String> readings) {

    /** The options, copied, with nothing null. */
    public ChartOptions {
        kind = kind == null ? ChartKind.NATAL : kind;
        lilith = lilith == null ? Lilith.NONE : lilith;
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
        private Lilith lilith = Lilith.NONE;
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
         * What the instant falls in: the rising sign's span and the eclipse seen from the place.
         *
         * @param on whether to read it
         * @return this builder
         */
        public Builder moment(boolean on) {
            return section(Native.TS_CHART_MOMENT, on);
        }

        /**
         * The Black Moon Lilith beside the nine, read back as
         * {@link Chart#lilith()} ({@code 03-design/lilith.md}).
         *
         * @param apogee {@link Lilith#MEAN} or {@link Lilith#OSCULATING}, or null for none
         * @return this builder
         * @throws IllegalArgumentException for {@link Lilith#NONE}, since null says that
         */
        public Builder lilith(Lilith apogee) {
            if (apogee == Lilith.NONE) {
                throw new IllegalArgumentException(
                        "lilith: expected Lilith.MEAN or Lilith.OSCULATING; pass null for none");
            }
            this.lilith = apogee == null ? Lilith.NONE : apogee;
            return this;
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
         * @param theme the request; null takes it away
         * @return this builder
         */
        public Builder theme(ThemePatch theme) {
            return json("theme_json", theme);
        }

        /**
         * Rules to answer over every chart.
         *
         * @param rules the request; null takes it away
         * @return this builder
         */
        public Builder rules(RuleRequest rules) {
            return json("rules_json", rules);
        }

        /**
         * Narrative plans to compose over every chart.
         *
         * @param interpret the request; null takes it away
         * @return this builder
         */
        public Builder interpret(PlanRequest interpret) {
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
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder varsha(VarshaRequest request) {
            return json("varsha_json", request);
        }

        /**
         * The gochar.
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder gochar(GocharRequest request) {
            return json("gochar_json", request);
        }

        /**
         * The transit hit.
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder hits(HitRequest request) {
            return json("hits_json", request);
        }

        /**
         * The Sade Sati.
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder sadeSati(SadeSatiRequest request) {
            return json("sade_sati_json", request);
        }

        /**
         * The KP.
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder kp(KpRequest request) {
            return json("kp_json", request);
        }

        /**
         * The dignities.
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder dignities(DignityRequest request) {
            return json("dignities_json", request);
        }

        /**
         * The fortitudes.
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder fortitudes(FortitudeRequest request) {
            return json("fortitudes_json", request);
        }

        /**
         * The lots.
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder lots(LotRequest request) {
            return json("lots_json", request);
        }

        /**
         * The considerations.
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder considerations(com.teispace.teistro.request.ConsiderationRules request) {
            return json("considerations_json", request);
        }

        /**
         * The perfection.
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder perfection(PerfectionRequest request) {
            return json("perfection_json", request);
        }

        /**
         * The progressions.
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder progressions(ProgressionsRequest request) {
            return json("progressions_json", request);
        }

        /**
         * The Western aspects.
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder westernAspects(WesternAspectRequest request) {
            return json("western_aspects_json", request);
        }

        /**
         * The aspect configurations, as
         * {@code ConfigurationRequest.builder().stelliumLeast(4).build()}: the aspects and orbs
         * the shapes are made of, Leo's nine under his orbs by default, and the bodies in one
         * tropical sign that make a stellium ({@code 03-design/western-configurations.md}).
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder westernConfigurations(ConfigurationRequest request) {
            return json("western_configurations_json", request);
        }

        /**
         * The synastry.
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder synastry(PartnerSynastry request) {
            return json("synastry_json", request);
        }

        /**
         * The parallels.
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder parallels(ParallelRequest request) {
            return json("parallels_json", request);
        }

        /**
         * The antiscia.
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder antiscia(AntisciaRequest request) {
            return json("antiscia_json", request);
        }

        /**
         * The midpoints.
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder midpoints(MidpointRequest request) {
            return json("midpoints_json", request);
        }

        /**
         * The Western houses.
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder westernHouses(HouseRequest request) {
            return json("western_houses_json", request);
        }

        /**
         * The harmonic.
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder harmonic(HarmonicRequest request) {
            return json("harmonic_json", request);
        }

        /**
         * The matching.
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder matching(MatchingRequest request) {
            return json("matching_json", request);
        }

        /**
         * The prashna.
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder prashna(PrashnaRequest request) {
            return json("prashna_json", request);
        }

        /**
         * The remedies.
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder remedies(RemedyRequest request) {
            return json("remedies_json", request);
        }

        /**
         * Lal Kitab, the 1952 edition: every member optional, as
         * {@code Map.of("cycle", Map.of("planet", Graha.VENUS, "year", 17), "year", 30)}
         * ({@code 03-design/lalkitab.md}).
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder lalkitab(LalKitabRequest request) {
            return json("lalkitab_json", request);
        }

        /**
         * The Sarvatobhadra chakra at each of the request's instants, as
         * {@code SarvatobhadraRequest.of(Map.of("instants", List.of(2460676.5), "motion", "SUN"))}
         * ({@code 03-design/gochar.md} §8).
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder sarvatobhadra(SarvatobhadraRequest request) {
            return json("sarvatobhadra_json", request);
        }

        /**
         * The tithi praveshas of every chart's years, as
         * {@code TithiPraveshaRequest.builder().through(40).build()}
         * ({@code 03-design/tithi-pravesha.md}).
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder tithiPravesha(TithiPraveshaRequest request) {
            return json("tithi_pravesha_json", request);
        }

        /**
         * One year's month charts, or one month's day charts, of every chart, as
         * {@code MasaPraveshaRequest.builder().year(30).month(4).build()}
         * ({@code 03-design/annual-chart.md}).
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder masaPravesha(MasaPraveshaRequest request) {
            return json("masa_pravesha_json", request);
        }

        /**
         * Every chart written as a dossier for a language model, as
         * {@code DossierRequest.builder().sections(List.of("GRAHAS", "DASHAS")).at(2460676.5).build()};
         * a section a request flag computes widens the request with it
         * ({@code 03-design/dossier.md}).
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder dossier(DossierRequest request) {
            return json("dossier_json", request);
        }

        /**
         * The rectification: each reading asked by its member, as
         * {@code Map.of("purify", Map.of("minutes", 30))}.
         *
         * @param request the request; null takes it away
         * @return this builder
         */
        public Builder rectification(RectificationRequest request) {
            return json("rectification_json", request);
        }

        /**
         * The options.
         *
         * @return the options
         */
        public ChartOptions build() {
            return new ChartOptions(kind, lilith, sections, vargas, dashas, drawings, readings);
        }
    }
}
