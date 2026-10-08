package com.teispace.teistro;

/**
 * A checked exception a provider written in Java threw, as its cause. An
 * unchecked one reaches the caller as itself; either way the library's
 * refusal is kept as a suppressed exception, so a stack trace shows what
 * the port made of it.
 */
public final class ProviderException extends RuntimeException {
    private static final long serialVersionUID = 1L;

    ProviderException(Exception cause) {
        super(cause.getMessage(), cause);
    }
}
