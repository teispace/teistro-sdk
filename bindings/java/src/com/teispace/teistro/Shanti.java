package com.teispace.teistro;

/**
 * A graha's shanti (BPHS ch. 84): its image, rik, japa in thousands, fuel, food, fee and gem, the
 * substance it rules (null for the nodes), its direction (null for Ketu, C345) and its mandala
 * place.
 *
 * @param graha The graha.
 * @param image Its image.
 * @param rik Its rik.
 * @param japaThousands Its japa, in thousands.
 * @param samidh Its fuel.
 * @param food Its food.
 * @param dakshina Its fee.
 * @param gem Its gem.
 * @param substance The substance it rules. May be null.
 * @param direction Its direction. May be null.
 * @param mandala Its place in the mandala.
 */
public record Shanti(
        Graha graha,
        String image,
        String rik,
        int japaThousands,
        String samidh,
        String food,
        String dakshina,
        String gem,
        String substance,
        Direction direction,
        String mandala) {
}
