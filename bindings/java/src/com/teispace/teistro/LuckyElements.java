package com.teispace.teistro;

/**
 * A sign lord's lucky elements, as the baseline engine gives them.
 *
 * @param colour the colour, a catalogue member a locale names ({@code colour.RED})
 * @param number the number
 * @param day the day
 * @param direction the direction
 */
public record LuckyElements(Colour colour, int number, Vara day, Direction direction) {}
