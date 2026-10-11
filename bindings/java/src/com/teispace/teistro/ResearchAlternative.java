package com.teispace.teistro;

/**
 * The direction a test looks in, declared before the data ({@code 03-design/research.md}).
 */
public enum ResearchAlternative {
    /** The rule holds more often than the null says. */
    GREATER,
    /** The rule holds less often than the null says. */
    LESS,
    /** Either way; the default. */
    TWO_SIDED;

    /**
     * The spelling the record carries.
     *
     * @return the key, as {@code "TWO_SIDED"}
     */
    public String key() {
        return name();
    }
}
