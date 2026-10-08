package com.teispace.teistro;

import java.util.List;

/**
 * What a node stands for, in Reader VI's order (C155).
 *
 * @param node The node.
 * @param conjoined The planets joined to it.
 * @param starLord Its star's lord.
 * @param aspecting The planets aspecting it.
 * @param signLord Its sign's lord.
 */
public record KpNodeAgency(
        Graha node,
        List<Graha> conjoined,
        Graha starLord,
        List<Graha> aspecting,
        Graha signLord) {
    /** The value, its lists copied and unmodifiable. */
    public KpNodeAgency {
        conjoined = List.copyOf(conjoined);
        aspecting = List.copyOf(aspecting);
    }
}
