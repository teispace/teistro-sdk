package com.teispace.teistro;

import java.util.List;

/**
 * One ruling planet, every reason it rules, and what rejects it.
 *
 * @param graha The ruling planet.
 * @param reasons Every reason it rules, the first the strongest.
 * @param retrograde Itself retrograde, which the Reader reads as delay and not rejection.
 * @param rejectedBy What rejects it under the settings; null when it stands. May be null.
 * @param rejectedBySub What would reject it under the other reading of C153. May be null.
 */
public record KpRuler(
        Graha graha,
        List<KpReason> reasons,
        boolean retrograde,
        KpRejection rejectedBy,
        KpRejection rejectedBySub) {
    /** The value, its lists copied and unmodifiable. */
    public KpRuler {
        reasons = List.copyOf(reasons);
    }
}
