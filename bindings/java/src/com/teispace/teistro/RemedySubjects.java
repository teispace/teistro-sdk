package com.teispace.teistro;

import java.util.List;

/**
 * Whom a remedy is for, in catalogue order and unranked, and the running antardasha's shanti; null
 * without {@code at}.
 *
 * @param subjects Whom a remedy is for.
 * @param antardasha The running antardasha's shanti. May be null.
 */
public record RemedySubjects(
        List<RemedySubject> subjects,
        AntardashaShanti antardasha) {
    /** The value, its lists copied and unmodifiable. */
    public RemedySubjects {
        subjects = List.copyOf(subjects);
    }
}
