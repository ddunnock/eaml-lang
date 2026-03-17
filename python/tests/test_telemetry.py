"""Tests for the telemetry hook system."""

from __future__ import annotations

import warnings

import pytest

from eaml_runtime.events import CallEndEvent, CallStartEvent
from eaml_runtime.telemetry import _fire, configure


def test_configure_registers_hook() -> None:
    called: list[object] = []
    configure(on_call_start=lambda e: called.append(e))
    event = CallStartEvent(provider="test", model_id="m")
    _fire("on_call_start", event)
    assert len(called) == 1


def test_fire_with_no_hook_is_noop() -> None:
    """_fire with no registered hook should not raise."""
    event = CallStartEvent(provider="test", model_id="m")
    _fire("on_call_start", event)  # Should not raise


def test_hook_receives_event() -> None:
    received: list[object] = []
    configure(on_call_start=lambda e: received.append(e))
    event = CallStartEvent(provider="anthropic", model_id="claude")
    _fire("on_call_start", event)
    assert received[0] is event


def test_hook_exception_swallowed() -> None:
    def bad_hook(event: object) -> None:
        raise ValueError("hook broke")

    configure(on_call_start=bad_hook)
    event = CallStartEvent(provider="test", model_id="m")
    with pytest.warns(UserWarning, match="hook broke"):
        _fire("on_call_start", event)


def test_configure_multiple_hooks() -> None:
    start_events: list[object] = []
    end_events: list[object] = []
    configure(
        on_call_start=lambda e: start_events.append(e),
        on_call_end=lambda e: end_events.append(e),
    )
    _fire("on_call_start", CallStartEvent(provider="a", model_id="m"))
    _fire("on_call_end", CallEndEvent(provider="a", model_id="m", duration=1.0))
    assert len(start_events) == 1
    assert len(end_events) == 1


def test_configure_overwrites_hook() -> None:
    calls_a: list[object] = []
    calls_b: list[object] = []
    configure(on_call_start=lambda e: calls_a.append(e))
    configure(on_call_start=lambda e: calls_b.append(e))
    _fire("on_call_start", CallStartEvent(provider="t", model_id="m"))
    assert len(calls_a) == 0
    assert len(calls_b) == 1


def test_fire_unknown_event_is_noop() -> None:
    """Firing an event name that has no hook should be silent."""
    _fire("on_nonexistent_event", object())


def test_hook_exception_does_not_propagate() -> None:
    """Even if the hook raises, the caller should not see the exception."""
    def bad_hook(event: object) -> None:
        raise RuntimeError("fatal")

    configure(on_call_start=bad_hook)
    # Suppress the expected warning to keep test output clean
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        _fire("on_call_start", CallStartEvent(provider="t", model_id="m"))
