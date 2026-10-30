"""The SDK failing inside itself, as this layer reports it: never expected,
and never silent (`docs/03-design/wasm-binding.md` §4, "A panic traps").

The C ABI turns a panic into the `INTERNAL` status before it can cross, so
a call refuses with it like any refusal. A host's crash reporter hears of
one through `on_internal_error`, with the status, its code, which kind it
was and the SDK's version: never the message, which may quote what the
call was given.
"""

from __future__ import annotations

import threading
from dataclasses import dataclass
from typing import Callable, Final, Literal

#: The stable code of the `INTERNAL` status, the same in every binding.
INTERNAL_CODE: Final = -10


@dataclass(frozen=True)
class InternalReport:
    """What a crash reporter hears of an `INTERNAL` refusal.

    The fields are all it hears: there is no message, on purpose.
    """

    sdk_version: str
    """The SDK that failed."""
    status: Literal["INTERNAL"] = "INTERNAL"
    """The status, as its key."""
    code: int = INTERNAL_CODE
    """`INTERNAL_CODE`."""
    kind: Literal["boundary"] = "boundary"
    """A panic the C ABI caught, the only kind outside a wasm module."""


_lock: Final = threading.Lock()
_hooks: Final[list[Callable[[InternalReport], object]]] = []


def on_internal_error(hook: Callable[[InternalReport], object]) -> Callable[[], None]:
    """Calls `hook` with every `InternalReport` from now on, from whichever
    thread was refused; the function it returns stops that.

    >>> stop = on_internal_error(print)
    >>> stop()
    """
    if not callable(hook):
        raise TypeError("on_internal_error takes a function")
    with _lock:
        if hook not in _hooks:
            _hooks.append(hook)

    def stop() -> None:
        with _lock:
            if hook in _hooks:
                _hooks.remove(hook)

    return stop


def report_internal(sdk_version: str) -> None:
    """Tells every hook. A hook that raises is not the caller's failure, so
    what it raises is dropped and the others still hear; only an `Exception`
    is, so an interrupt still interrupts."""
    report = InternalReport(sdk_version=sdk_version)
    with _lock:
        hooks = tuple(_hooks)
    for hook in hooks:
        try:
            hook(report)
        except Exception:  # pylint: disable=broad-except
            # A crash reporter's own failure is its own.
            pass
