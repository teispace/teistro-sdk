package com.teispace.teistro;

/**
 * Where a self-matched study reads each person besides their event ({@code 03-design/research.md}
 * §1.4): the life cut into windows of {@code windowDays} from birth, the event's window read every
 * {@code stepDays} (time-stratified referents). The windows are fixed before the event is known, and
 * none starts before birth.
 *
 * <pre>{@code
 * ResearchReferents year = new ResearchReferents(365.25, 30);
 * }</pre>
 *
 * @param windowDays each window's length in days
 * @param stepDays the days between a person's instants; a window holds at most 1000
 */
public record ResearchReferents(double windowDays, double stepDays) {}
