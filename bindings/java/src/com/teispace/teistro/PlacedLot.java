package com.teispace.teistro;

/**
 * One lot and where it fell.
 *
 * @param lot the lot
 * @param place where it fell
 */
public record PlacedLot(Lot lot, LotPlace place) {}
