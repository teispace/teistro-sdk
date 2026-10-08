package com.teispace.teistro;

/**
 * The five exceptions of VI.32 to 33 that lift a bad Bhakoot, each a clause that holds or not.
 *
 * @param oneLord One lord rules both signs.
 * @param lordsFriends The sign lords are each other's friends.
 * @param navamshaLordsFriends The navamsha lords are one or each other's friends.
 * @param taraPure The tara is pure both ways.
 * @param vashya One sign is vashya to the other.
 */
public record BhakootExceptions(
        boolean oneLord,
        boolean lordsFriends,
        boolean navamshaLordsFriends,
        boolean taraPure,
        boolean vashya) {
}
