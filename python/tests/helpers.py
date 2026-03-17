"""Shared mock providers for eaml_runtime tests."""

from __future__ import annotations

from typing import Any

from eaml_runtime.providers import Provider


class MockProvider(Provider):
    """A test provider that returns controlled responses in order."""

    def __init__(self, responses: list[str] | None = None) -> None:
        self.responses = list(responses) if responses else []
        self.calls: list[dict[str, Any]] = []
        self._idx = 0

    async def send_prompt(
        self,
        messages: list[dict[str, str]],
        model_id: str,
        *,
        temperature: float | None = None,
        max_tokens: int | None = None,
    ) -> str:
        self.calls.append({
            "messages": [dict(m) for m in messages],
            "model_id": model_id,
            "temperature": temperature,
            "max_tokens": max_tokens,
        })
        if self._idx < len(self.responses):
            resp = self.responses[self._idx]
            self._idx += 1
            return resp
        return "{}"


class ErrorProvider(Provider):
    """A test provider that always raises the given exception."""

    def __init__(self, exc: Exception) -> None:
        self.exc = exc

    async def send_prompt(
        self,
        messages: list[dict[str, str]],
        model_id: str,
        *,
        temperature: float | None = None,
        max_tokens: int | None = None,
    ) -> str:
        raise self.exc
