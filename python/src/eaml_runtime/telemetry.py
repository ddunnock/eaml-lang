"""Global telemetry hook registry."""

from __future__ import annotations

import warnings
from typing import Callable

_hooks: dict[str, Callable[..., None] | None] = {
    "on_call_start": None,
    "on_call_end": None,
    "on_validation_failure": None,
    "on_tool_call": None,
}


def configure(**kwargs: Callable[..., None] | None) -> None:
    """Register telemetry hooks by event name.

    Accepts keyword arguments matching hook names: on_call_start, on_call_end,
    on_validation_failure, on_tool_call. Unknown keys are silently ignored.
    """
    for key, value in kwargs.items():
        if key in _hooks:
            _hooks[key] = value


def _fire(event_name: str, event: object) -> None:
    """Fire a telemetry event. Swallows hook exceptions with warnings.warn()."""
    hook = _hooks.get(event_name)
    if hook is not None:
        try:
            hook(event)
        except Exception as exc:
            warnings.warn(
                f"Telemetry hook '{event_name}' raised: {exc}",
                UserWarning,
                stacklevel=2,
            )


def _reset() -> None:
    """Reset all hooks to None. Used in tests."""
    for key in _hooks:
        _hooks[key] = None
