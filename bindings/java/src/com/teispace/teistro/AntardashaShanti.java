package com.teispace.teistro;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

/**
 * The running antardasha's shanti and whether each condition holds, in the printed order: a null
 * element where the verse leaves it open.
 *
 * @param shanti The shanti.
 * @param holds Whether each condition holds, null where the verse leaves it open.
 */
public record AntardashaShanti(
        DashaShanti shanti,
        List<Boolean> holds) {
    /** The value, its lists copied and unmodifiable. */
    public AntardashaShanti {
        holds = Collections.unmodifiableList(new ArrayList<>(holds));
    }
}
