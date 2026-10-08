package com.teispace.teistro;

import java.util.LinkedHashMap;
import java.util.Map;

/**
 * The rules a chart's lots were read under, every field filled.
 *
 * @param sectRule the rule that chose the sect
 * @param fortune how Fortune is taken by night (C221)
 */
public record LotRules(SectRule sectRule, FortuneRule fortune) {
    /**
     * The rules as a request writes them, to hand back as a lots request.
     *
     * @return the request record, for {@link Json#write}
     */
    public Map<String, Object> request() {
        Map<String, Object> out = new LinkedHashMap<>();
        out.put("sectRule", sectRule);
        out.put("fortune", fortune);
        return out;
    }
}
