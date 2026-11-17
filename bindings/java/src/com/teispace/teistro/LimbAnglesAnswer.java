package com.teispace.teistro;

import com.teispace.teistro.record.Provenance;
import java.util.List;

/**
 * What {@link AlmanacArea#limbAngles} answers.
 *
 * @param value one for each instant asked, in its order
 * @param provenance what computed it, and the hash of {@code value}
 */
public record LimbAnglesAnswer(List<LimbAngles> value, Provenance provenance) {}
