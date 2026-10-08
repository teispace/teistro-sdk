package com.teispace.teistro;

import java.util.List;

/**
 * What BPHS prints for one antardasha: where, the conditions any of which brings the evil, and the
 * rites, all of which are prescribed.
 *
 * @param mahadasha The mahadasha's lord.
 * @param antardasha The antardasha's lord.
 * @param chapter The chapter.
 * @param verses The verses.
 * @param page The page.
 * @param conditions The conditions, any of which brings the evil.
 * @param remedies The rites, all of which are prescribed.
 */
public record DashaShanti(
        Graha mahadasha,
        Graha antardasha,
        int chapter,
        String verses,
        int page,
        List<String> conditions,
        List<String> remedies) {
    /** The value, its lists copied and unmodifiable. */
    public DashaShanti {
        conditions = List.copyOf(conditions);
        remedies = List.copyOf(remedies);
    }
}
