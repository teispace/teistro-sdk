package com.teispace.teistro;

import java.util.function.Supplier;

/**
 * A value computed once, the first time it is asked for, and kept: what a
 * batch parses from its blob is parsed once however many charts read it.
 * Safe to share between threads; two that ask at once compute it once.
 */
final class Lazy<T> implements Supplier<T> {
    private final Supplier<T> compute;
    private volatile T value;

    private Lazy(Supplier<T> compute) {
        this.compute = compute;
    }

    static <T> Lazy<T> of(Supplier<T> compute) {
        return new Lazy<>(compute);
    }

    @Override
    public T get() {
        T found = value;
        if (found == null) {
            synchronized (this) {
                found = value;
                if (found == null) {
                    found = compute.get();
                    value = found;
                }
            }
        }
        return found;
    }
}
