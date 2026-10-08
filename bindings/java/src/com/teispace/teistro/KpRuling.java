package com.teispace.teistro;

import java.util.List;

/**
 * The ruling planets of a moment, and the settings they were read under.
 *
 * @param rulers The rulers, strongest first.
 * @param rules The settings they were read under.
 */
public record KpRuling(
        List<KpRuler> rulers,
        KpRulingRules rules) {
    /** The value, its lists copied and unmodifiable. */
    public KpRuling {
        rulers = List.copyOf(rulers);
    }

    /**
     * The rulers that stand, in order.
     *
     * @return the rulers no planet rejects
     */
    public List<Graha> accepted() {
        return rulers.stream().filter(ruler -> ruler.rejectedBy() == null).map(KpRuler::graha).toList();
    }
}
