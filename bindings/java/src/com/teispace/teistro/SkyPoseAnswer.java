package com.teispace.teistro;

import com.teispace.teistro.record.Provenance;

/**
 * What {@link AlmanacArea#skyPose} answers.
 *
 * @param value the sky and where each of its points falls
 * @param provenance what computed it, and the hash of {@code value}
 */
public record SkyPoseAnswer(SkyPose value, Provenance provenance) {}
