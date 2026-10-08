package com.teispace.teistro;

import java.util.List;

/**
 * One word of a name.
 *
 * @param text the word as written
 * @param letters its letters and their values
 * @param total the letters added
 * @param reduction the total reduced
 */
public record WordNumber(String text, List<NumerologyLetter> letters, int total, Reduction reduction) {
    /** Keeps the list unmodifiable. */
    public WordNumber {
        letters = List.copyOf(letters);
    }
}
