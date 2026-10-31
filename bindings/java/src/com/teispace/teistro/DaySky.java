package com.teispace.teistro;

/**
 * The sky of one day at one place ({@code 03-design/day-sky.md}).
 *
 * @param from the window's start, Julian day, UT1 read as UTC
 * @param to the window's end; an event here is the next window's
 * @param stepMinutes the step asked, minutes
 * @param sun the Sun
 * @param moon the Moon
 */
public record DaySky(double from, double to, int stepMinutes, BodySky sun, BodySky moon) {}
