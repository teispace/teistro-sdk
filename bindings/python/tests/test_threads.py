"""Threads may share a context (`03-design/python-binding.md`).

A native context is one thread's at a time, and ctypes lets go of the GIL
for every call, so two threads calling one context entered it at once:
one's answer could be read through the other's call, and a refusal's
message was the context's last error, which the other thread may have
written. Each call now holds the context's lock for as long as its answer
is read. Here several threads ask one context for answers each knows
beforehand, interleaved with refusals, and every answer must be its own.
"""

from __future__ import annotations

import threading

from teistro import Body, TeistroError
from tests.support import LOCALE, PROFILE, WithLibrary

THREADS = 8
ROUNDS = 40
# Long enough a call that the threads' calls overlap.
INSTANTS = 400


class SharedContextTest(WithLibrary):
    def test_threads_sharing_a_context_each_read_their_own_answers(self) -> None:
        with self.teistro.context(profile=PROFILE, locale=LOCALE, test_provider=True) as ctx:
            grids = [[2451545.0 + 37.0 * k + 0.5 * n for n in range(INSTANTS)] for k in range(THREADS)]
            expected = [
                ctx.positions(instants=grid, bodies=[Body.SUN, Body.MOON]).at(INSTANTS - 1, 1).longitude
                for grid in grids
            ]
            profile = ctx.profile
            start = threading.Barrier(THREADS)
            faults: list[str] = []

            def ask(k: int) -> None:
                try:
                    rounds(k)
                except Exception as fault:  # a fault in a thread is otherwise lost
                    faults.append(f"thread {k} raised {fault!r}")

            def rounds(k: int) -> None:
                start.wait()
                for _ in range(ROUNDS):
                    moon = ctx.positions(instants=grids[k], bodies=[Body.SUN, Body.MOON]).at(INSTANTS - 1, 1)
                    if moon.longitude != expected[k]:
                        faults.append(f"thread {k} read {moon.longitude}, not {expected[k]}")
                    if ctx.profile != profile:
                        faults.append(f"thread {k} read the profile {ctx.profile!r}")
                    unknown = f"graha.UNKNOWN_TO_THREAD_{k}"
                    try:
                        ctx.inner.key_parse(unknown)
                        faults.append(f"thread {k}: {unknown} was parsed")
                    except TeistroError as refusal:
                        if unknown.split(".")[1] not in str(refusal):
                            faults.append(f"thread {k} was refused with another's error: {refusal}")

            threads = [threading.Thread(target=ask, args=(k,)) for k in range(THREADS)]
            for thread in threads:
                thread.start()
            for thread in threads:
                thread.join()
            self.assertEqual(faults[:5], [])
