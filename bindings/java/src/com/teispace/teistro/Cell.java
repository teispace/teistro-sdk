package com.teispace.teistro;

/**
 * One body at one instant, as the positions grid holds it.
 *
 * @param longitude the longitude, in the grid's frame
 * @param latitude the latitude
 * @param distance the distance
 * @param longitudeSpeed the longitude's speed
 * @param latitudeSpeed the latitude's speed
 * @param distanceSpeed the distance's speed
 * @param status the cell's status, as the boundary reports it
 * @param source which provider answered the cell
 */
public record Cell(
        double longitude, double latitude, double distance, double longitudeSpeed, double latitudeSpeed,
        double distanceSpeed, int status, long source) {}
