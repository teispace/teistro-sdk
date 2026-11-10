package com.teispace.teistro;

import java.util.Map;

/**
 * A sankalpa said by a tradition's template ({@code 03-design/sankalpa.md} §5).
 *
 * @param tradition the tradition: the template's message key
 * @param locale the locale it was said in
 * @param params the slots the template was given, as {@code intl.render} takes them, to say it again in
 *     another locale
 * @param text the words
 */
public record SankalpaWords(String tradition, String locale, Map<?, ?> params, String text) {}
