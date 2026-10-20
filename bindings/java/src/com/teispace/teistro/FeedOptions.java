package com.teispace.teistro;

/**
 * What an iCalendar feed says of itself ({@code 03-design/icalendar.md}): its name, the product
 * that wrote it, how often a subscriber fetches it again, and the instant every event is stamped
 * with. Each null leaves the SDK's own: "Festivals", the SDK's {@code PRODID}, a day, and now.
 *
 * <pre>{@code
 * FeedOptions options = FeedOptions.defaults().withName("Nepal 2026").withStampUnixSeconds(1_767_225_600L);
 * }</pre>
 *
 * @param name the calendar's name, or null for "Festivals"
 * @param product the {@code PRODID}, or null for the SDK's own
 * @param refreshMinutes how often a subscriber fetches it again, or null for a day
 * @param noRefresh whether to write no refresh interval at all
 * @param stampUnixSeconds every event's {@code DTSTAMP} in seconds since 1970, or null for now
 */
public record FeedOptions(String name, String product, Integer refreshMinutes, boolean noRefresh,
        Long stampUnixSeconds) {
    /**
     * The SDK's own for every field.
     *
     * @return options that leave everything to the SDK
     */
    public static FeedOptions defaults() {
        return new FeedOptions(null, null, null, false, null);
    }

    /**
     * With another name.
     *
     * @param name the calendar's name
     * @return these options with that name
     */
    public FeedOptions withName(String name) {
        return new FeedOptions(name, product, refreshMinutes, noRefresh, stampUnixSeconds);
    }

    /**
     * With another {@code PRODID}.
     *
     * @param product as {@code -//Owner//Product//EN}
     * @return these options with that product
     */
    public FeedOptions withProduct(String product) {
        return new FeedOptions(name, product, refreshMinutes, noRefresh, stampUnixSeconds);
    }

    /**
     * With another refresh interval.
     *
     * @param minutes how often a subscriber fetches the feed again
     * @return these options with that interval
     */
    public FeedOptions withRefreshMinutes(int minutes) {
        return new FeedOptions(name, product, minutes, false, stampUnixSeconds);
    }

    /**
     * With no refresh interval, leaving it to the app.
     *
     * @return these options writing none
     */
    public FeedOptions withoutRefresh() {
        return new FeedOptions(name, product, null, true, stampUnixSeconds);
    }

    /**
     * With another stamp.
     *
     * @param seconds every event's {@code DTSTAMP}, in seconds since 1970-01-01T00:00:00Z
     * @return these options with that stamp
     */
    public FeedOptions withStampUnixSeconds(long seconds) {
        return new FeedOptions(name, product, refreshMinutes, noRefresh, seconds);
    }
}
