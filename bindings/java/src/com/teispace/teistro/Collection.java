package com.teispace.teistro;

/**
 * A heavier planet both significators apply to (p. 112); who must receive
 * whom is C233.
 *
 * @param collector the heavier planet
 * @param fromQuerent the querent's significator's application to it
 * @param fromQuesited the quesited's significator's application to it
 * @param collectorInQuerent the querent's dignities the collector stands in
 * @param collectorInQuesited the quesited's dignities the collector stands in
 * @param querentInCollector the collector's dignities the querent's stands in
 * @param quesitedInCollector the collector's dignities the quesited's stands in
 */
public record Collection(
        Graha collector, ContactAhead fromQuerent, ContactAhead fromQuesited, EssentialDignity collectorInQuerent,
        EssentialDignity collectorInQuesited, EssentialDignity querentInCollector,
        EssentialDignity quesitedInCollector) {}
