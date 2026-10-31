package com.teispace.teistro;

/**
 * The degree of a line counted out, the common forebear first (<i>Dharmasindhu</i> p. 299).
 *
 * @param observer the observer's number from the common forebear, who is 1
 * @param person the person's number
 * @param reach the last person who is a sapinda here, the living forebears added
 */
public record KinDegree(int observer, int person, int reach) {}
