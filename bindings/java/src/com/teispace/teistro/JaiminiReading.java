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
 * @param sthiraKarakas The sthira karakas of BPHS ch. 32 vv. 18 to 20, ten in the table's order: the
 *     father's and the mother's the stronger of their pair (C408, C409), and each says how it was chosen.
 * @param karakaBhavas The karaka bhavas of BPHS ch. 32 vv. 22 to 24, seven: the house counted from a
 *     graha's sign, whole signs in the rasi chart, and the sign it falls in.
 */
public record JaiminiReading(
        Karakamsha karakamsha,
        Brahma brahma,
        List<Rashi> grahaArudhas,
        List<SthiraKaraka> sthiraKarakas,
        List<KarakaBhava> karakaBhavas) {
    /** The value, its lists copied and unmodifiable. */
    public JaiminiReading {
        grahaArudhas = Collections.unmodifiableList(new ArrayList<>(grahaArudhas));
        sthiraKarakas = List.copyOf(sthiraKarakas);
        karakaBhavas = List.copyOf(karakaBhavas);
    }
}
