package com.teispace.teistro;

/** {@code sky.frame()}: the coordinate conventions a request is expressed in. */
public final class FrameArea {
    private final Teistro teistro;

    FrameArea(Teistro teistro) {
        this.teistro = teistro;
    }

    /**
     * The SDK's canonical frame: apparent geocentric ecliptic of date, tropical.
     *
     * @return the frame
     */
    public Frame canonical() {
        return teistro.canonicalFrame();
    }

    /**
     * A frame's fields as the bits a position request carries.
     *
     * @param frame the frame
     * @return its bits
     */
    public long pack(Frame frame) {
        return teistro.packFrame(frame);
    }

    /**
     * The frame a packed set of bits describes.
     *
     * @param bits the bits
     * @return the frame
     */
    public Frame unpack(long bits) {
        return teistro.unpackFrame(bits);
    }
}
