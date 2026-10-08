package com.teispace.teistro;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

/**
 * A chart's Jaimini significators, read under the settings' {@code jaimini} group.
 *
 * @param karakamsha The karakamsha, with every graha's house from it in both charts.
 * @param brahma The Brahma graha, or why there is none.
 * @param grahaArudhas Each graha's arudha, the Sun to Ketu (BPHS ch. 29 vv. 6 and 7), under
 *     {@code jaimini.graha_arudha_exception}; a null element for a node that owns no sign under
 *     {@code jaimini.node_co_lordship}.
 */
public record JaiminiReading(
        Karakamsha karakamsha,
        Brahma brahma,
        List<Rashi> grahaArudhas) {
    /** The value, its lists copied and unmodifiable. */
    public JaiminiReading {
        grahaArudhas = Collections.unmodifiableList(new ArrayList<>(grahaArudhas));
    }
}
