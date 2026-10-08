package com.teispace.teistro;

/**
 * One of the baseline engine's weights: what it measured (a dimension such
 * as {@code TARA_BALA}), by how much, and the graha it read, if one.
 *
 * @param dimension what it measured
 * @param weight by how much
 * @param graha the graha it read; may be null
 */
public record MuhurtaFactor(String dimension, int weight, Graha graha) {}
