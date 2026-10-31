package com.teispace.teistro;

import java.util.List;

/**
 * A household's periods: each observer's ({@code 03-design/ashauch.md} §11).
 *
 * @param observers each observer, in the request's order
 */
public record HouseholdAshauch(List<HouseholdPeriods> observers) {
    /** Keeps the lists unmodifiable. */
    public HouseholdAshauch {
        observers = List.copyOf(observers);
    }
}
