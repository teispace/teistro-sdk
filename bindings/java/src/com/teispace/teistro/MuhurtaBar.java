package com.teispace.teistro;

/**
 * A bar that struck a muhurta window: a clause's tag, which bars every
 * clause of that kind, or one clause. Python answers either as it stands
 * (a {@code str} or a clause); exactly one of the two is set here.
 *
 * @param tag the tag barring every clause of a kind; may be null, when one clause barred it
 * @param clause the one clause that barred it; may be null, when a tag did
 */
public record MuhurtaBar(String tag, MuhurtaClauseKind clause) {
    /** Holds exactly one of the two. */
    public MuhurtaBar {
        if ((tag == null) == (clause == null)) {
            throw new IllegalArgumentException("a muhurta bar is a tag or a clause, exactly one of the two");
        }
    }

    /**
     * The bar as a request's {@code bars} writes it: the tag, or the
     * clause's tag and fields.
     *
     * @return the request value, for {@link Json#write}
     */
    public Object request() {
        return tag != null ? tag : clause.request();
    }

    static MuhurtaBar read(Object raw) {
        return raw instanceof String tag ? new MuhurtaBar(tag, null) : new MuhurtaBar(null, MuhurtaClauseKind.read(raw));
    }
}
