package com.teispace.teistro;

import java.util.List;

import com.teispace.teistro.record.Provenance;

/**
 * What {@code verify.claims} answers: every claim a text makes of the time,
 * in the text's order, sealed with the provenance of the sankalpa they
 * were judged against.
 *
 * @param claims the claims
 * @param provenance what computed the sankalpa
 */
public record ClaimsAnswer(List<Claim> claims, Provenance provenance) {

    /**
     * One claim, and what it was found to be.
     *
     * @param start where the span naming the member begins, counted in code points, not UTF-16 units
     * @param end where it ends, exclusive, in the same count
     * @param text the span's text
     * @param kind the kind claimed, or null on a claim ambiguous between kinds
     * @param graha the graha whose sign or nakshatra is claimed, or null
     * @param claimed every member the span may name
     * @param verdict {@code HELD}, {@code CONTRADICTED} or {@code UNREAD}
     * @param computed the member the SDK computes, on a claim judged, or null
     * @param reason why the claim was not judged, on one unread, or null
     */
    public record Claim(int start, int end, String text, String kind, String graha, List<String> claimed,
            String verdict, String computed, String reason) {}
}
