package com.teispace.teistro;

import java.util.List;

import com.teispace.teistro.record.Provenance;

/**
 * A study's counts ({@code 03-design/research.md}): how often each rule holds in each group, with the
 * provenance whose {@code inputHash} is the study's pre-registration.
 *
 * @param rows one row per predicate, in the request's order
 * @param provenance how it was produced
 */
public record ResearchCounts(List<Row> rows, Provenance provenance) {
    /**
     * A predicate's charts in one group. The unreadable and the unstable are left out of every
     * denominator.
     *
     * @param present read, and the predicate holds
     * @param absent read, and it does not
     * @param unreadable not read
     * @param unstable different answers inside the time uncertainty
     */
    public record Group(int present, int absent, int unreadable, int unstable) {
    }

    /**
     * One predicate's charts in each group.
     *
     * @param predicate the rule's key
     * @param counts its charts in each group, groups in index order
     */
    public record Row(String predicate, List<Group> counts) {
    }
}
