package com.teispace.teistro;

import java.util.List;
import java.util.Map;

/**
 * Why a window is good, in words ({@code 03-design/muhurta-reasons.md}): a short line and the
 * longer list, each item an intent a voice pack words. {@code intl().render(item.intent(),
 * item.params())} says one.
 *
 * @param line the "why this date" line
 * @param reasons the list: {@code FOR}, then {@code DESPITE}, then {@code AVOIDED}
 */
public record MuhurtaReasons(Line line, List<Reason> reasons) {
    /** Keeps the list unmodifiable. */
    public MuhurtaReasons {
        reasons = List.copyOf(reasons);
    }

    /**
     * The line.
     *
     * @param intent {@code sdk.voice.muhurta.line}
     * @param params its slots: the qualifying {@code limbs}, their {@code count}, how many doshas
     *     were {@code avoided}, how many cautions remain ({@code despite}) and the {@code variant}
     */
    public record Line(String intent, Map<String, Object> params) {
        /** Keeps the slots unmodifiable. */
        public Line {
            params = Map.copyOf(params);
        }
    }

    /**
     * One reason.
     *
     * @param intent the intent, as {@code sdk.voice.muhurta.avoided.MARS_IN_EIGHTH}
     * @param params its slots: the clause's members and a {@code variant}
     * @param side {@code FOR}, {@code DESPITE} or {@code AVOIDED}
     * @param key the clause's key, as {@code MARS_IN_EIGHTH}
     * @param clause for {@code FOR} and {@code DESPITE}, the clause's index in the window's
     *     clauses; null for {@code AVOIDED}, which says an absence
     */
    public record Reason(String intent, Map<String, Object> params, String side, String key,
            Integer clause) {
        /** Keeps the slots unmodifiable. */
        public Reason {
            params = Map.copyOf(params);
        }
    }

    /** One window's reasons, as the answer writes them. */
    static MuhurtaReasons read(Object raw) {
        Map<?, ?> one = Reads.object(raw);
        Map<?, ?> line = Reads.object(one, "line");
        return new MuhurtaReasons(
                new Line(Reads.string(line, "intent"), JsonRead.object(Reads.field(line, "params"))),
                Reads.each(one, "reasons", r -> {
                    Map<?, ?> reason = Reads.object(r);
                    return new Reason(
                            Reads.string(reason, "intent"),
                            JsonRead.object(Reads.field(reason, "params")),
                            Reads.string(reason, "side"),
                            Reads.string(reason, "key"),
                            Reads.optionalInteger(reason, "clause"));
                }));
    }
}
