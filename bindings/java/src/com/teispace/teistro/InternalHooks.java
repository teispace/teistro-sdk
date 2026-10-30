package com.teispace.teistro;

import java.util.Objects;
import java.util.Set;
import java.util.concurrent.CopyOnWriteArraySet;
import java.util.function.Consumer;

import com.teispace.teistro.ffi.Native;

/**
 * The SDK failing inside itself, as this layer reports it: never expected,
 * and never silent ({@code docs/03-design/wasm-binding.md} §4, "A panic
 * traps"). The C ABI turns a panic into the {@code INTERNAL} status before it
 * can cross, so a call refuses with it like any refusal, and the hooks hear
 * of it here.
 */
final class InternalHooks {
    private static final Set<Consumer<InternalReport>> HOOKS = new CopyOnWriteArraySet<>();

    private InternalHooks() {}

    /** Adds a hook; running what this returns removes it. */
    static Runnable add(Consumer<InternalReport> hook) {
        Consumer<InternalReport> held = Objects.requireNonNull(hook, "hook");
        HOOKS.add(held);
        return () -> HOOKS.remove(held);
    }

    /**
     * Hands back a refusal the boundary answered, having told every hook if
     * it was {@code INTERNAL}. What a hook throws is dropped, since a crash
     * reporter's own failure is not the caller's, and the others still hear.
     */
    static TeistroException told(TeistroException refusal) {
        if (refusal.status() == Status.INTERNAL) {
            InternalReport report = new InternalReport(
                    Status.INTERNAL.key(), Teistro.INTERNAL_CODE, "boundary", Native.GENERATED_SDK_VERSION);
            for (Consumer<InternalReport> hook : HOOKS) {
                try {
                    hook.accept(report);
                } catch (Throwable dropped) {
                    continue;
                }
            }
        }
        return refusal;
    }
}
