package com.teispace.teistro;

/**
 * Where a significator stands.
 *
 * @param planet the significator
 * @param house its house, 1 to 12
 * @param dignity the dignities it holds there
 */
public record SignificatorPlace(Graha planet, int house, EssentialDignity dignity) {}
