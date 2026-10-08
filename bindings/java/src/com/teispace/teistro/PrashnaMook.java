package com.teispace.teistro;

/**
 * What an unspoken question is about: the graha whose house was read, the person under
 * {@code SHATPANCHASHIKA} (null otherwise), and the class of the thing thought of
 * ({@code MINERAL}, {@code ROOT} or {@code LIVING}).
 *
 * @param rule The rule it was read under.
 * @param graha The graha whose house was read.
 * @param tie Whether that graha won a tie in strength.
 * @param house The house read.
 * @param person The person, under {@code SHATPANCHASHIKA}. May be null.
 * @param thought The class of the thing thought of.
 */
public record PrashnaMook(
        String rule,
        Graha graha,
        boolean tie,
        int house,
        String person,
        String thought) {
}
