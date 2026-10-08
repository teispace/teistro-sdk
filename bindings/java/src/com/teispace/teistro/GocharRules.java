package com.teispace.teistro;

/**
 * The readings of the nodes a transit was judged under, the settings' {@code gochar} group.
 *
 * @param nodeVedha The nodes' vedha (C136).
 * @param nodeObstruction Whom the nodes obstruct (C137, C140).
 * @param ashtakavargaGoodFrom How many bindus make a transit good by the Ashtakavarga (C141).
 */
public record GocharRules(
        NodeVedha nodeVedha,
        NodeObstruction nodeObstruction,
        AshtakavargaGoodFrom ashtakavargaGoodFrom) {
}
