"""The SDK failing inside itself (SR-52): an `INTERNAL` refusal at the
boundary reaches a host's crash reporter, without the message.

No build of the library can be made to panic on request, so the real
library's own records are classified here at the layer that classifies
them: `_raise` over a context's real last error and `_refuse` over a call
with none, given the `INTERNAL` status a panic answers with.
"""

from __future__ import annotations

import unittest

from teistro import (
    INTERNAL_CODE,
    InternalReport,
    Status,
    TeistroError,
    on_internal_error,
)
from teistro._ffi import _refuse  # pylint: disable=protected-access
from teistro._internal import report_internal

from tests.support import WithLibrary


class AHook(unittest.TestCase):
    def test_it_hears_the_status_code_kind_and_version_and_no_message(self) -> None:
        heard: list[InternalReport] = []
        stop = on_internal_error(heard.append)
        report_internal("1.2.3")
        stop()
        report_internal("1.2.3")
        self.assertEqual(heard, [InternalReport(sdk_version="1.2.3")])
        report = heard[0]
        self.assertEqual(
            (report.status, report.code, report.kind, report.sdk_version),
            ("INTERNAL", -10, "boundary", "1.2.3"),
        )
        self.assertFalse(hasattr(report, "message"))
        with self.assertRaises(AttributeError):
            report.kind = "trap"  # type: ignore[assignment,misc]

    def test_a_hook_that_raises_is_dropped_and_the_others_still_hear(self) -> None:
        def broken(_: InternalReport) -> None:
            raise ValueError("a broken reporter")

        heard: list[InternalReport] = []
        stop_broken = on_internal_error(broken)
        stop = on_internal_error(heard.append)
        report_internal("1.2.3")
        stop_broken()
        stop()
        self.assertEqual(len(heard), 1)

    def test_a_hook_added_twice_hears_once_and_stopping_twice_is_allowed(self) -> None:
        heard: list[InternalReport] = []
        stop = on_internal_error(heard.append)
        on_internal_error(heard.append)
        report_internal("1.2.3")
        stop()
        stop()
        report_internal("1.2.3")
        self.assertEqual(len(heard), 1)

    def test_a_hook_must_be_a_function(self) -> None:
        with self.assertRaises(TypeError):
            on_internal_error("not a function")  # type: ignore[arg-type]


class TheBoundary(WithLibrary):
    def setUp(self) -> None:
        self.heard: list[InternalReport] = []
        self.addCleanup(on_internal_error(self.heard.append))

    def test_the_status_is_the_code_every_binding_has(self) -> None:
        self.assertEqual(Status.INTERNAL.value, INTERNAL_CODE)
        self.assertEqual(INTERNAL_CODE, -10)

    def test_an_internal_refusal_of_a_context_tells_the_hook(self) -> None:
        with self.teistro.context(test_provider=True) as context:
            with self.assertRaises(TeistroError) as refused:
                context.inner._raise(Status.INTERNAL)  # pylint: disable=protected-access
        self.assertEqual(refused.exception.status, Status.INTERNAL)
        self.assertEqual(self.heard, [InternalReport(sdk_version=self.teistro.version)])

    def test_an_internal_refusal_with_no_context_tells_the_hook(self) -> None:
        with self.assertRaises(TeistroError) as refused:
            _refuse(self.teistro.library, Status.INTERNAL, None)
        self.assertEqual(refused.exception.message, "internal error")
        self.assertEqual(len(self.heard), 1)

    def test_a_closed_contexts_internal_refusal_still_tells_the_hook(self) -> None:
        context = self.teistro.context(test_provider=True)
        context.close()
        with self.assertRaises(TeistroError):
            context.inner._raise(Status.INTERNAL)  # pylint: disable=protected-access
        self.assertEqual(len(self.heard), 1)

    def test_a_real_refusal_of_any_other_status_tells_no_one(self) -> None:
        with self.assertRaises(TeistroError):
            self.teistro.context(profile="no-such-profile", test_provider=True)
        with self.assertRaises(TeistroError):
            _refuse(self.teistro.library, Status.INVALID_ARG, None)
        self.assertEqual(self.heard, [])
