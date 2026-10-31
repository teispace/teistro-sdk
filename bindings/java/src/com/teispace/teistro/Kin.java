package com.teispace.teistro;

import com.teispace.teistro.request.AshauchRelation;
import com.teispace.teistro.request.KinKeeps;

/**
 * What {@link AlmanacArea#ashauchKin} answers ({@code 03-design/ashauch.md} §10): the relation an
 * ashauch asks for, what is kept, and why.
 *
 * @param relation the relation to ask an ashauch with, when a period is kept; null otherwise
 * @param keeps what is kept
 * @param degree the degree, when the path is one line; null otherwise
 * @param withinGirlsReach for an unmarried daughter of the line, whether the observer stands within
 *     the persons her bands name; null otherwise
 * @param clause the text and page
 * @param table the table's name
 */
public record Kin(AshauchRelation relation, KinKeeps keeps, KinDegree degree, Boolean withinGirlsReach,
        String clause, String table) {}
