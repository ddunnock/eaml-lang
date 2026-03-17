"""Ollama provider adapter."""

from __future__ import annotations

import os
from typing import Any

from eaml_runtime.errors import EamlConfigError, EamlProviderError
from eaml_runtime.providers import Provider


class OllamaProvider(Provider):
    """Provider adapter for Ollama local models via OpenAI-compatible API."""

    _client: Any = None

    def _get_client(self) -> Any:
        """Get or create the httpx async client."""
        if self._client is None:
            try:
                import httpx  # noqa: F811
            except ImportError:
                raise EamlConfigError(
                    "Install httpx: pip install httpx"
                ) from None
            self._client = httpx.AsyncClient(timeout=120.0)
        return self._client

    def _base_url(self) -> str:
        """Get the Ollama API base URL from environment or default."""
        return os.environ.get("OLLAMA_BASE_URL", "http://localhost:11434")

    async def send_prompt(
        self,
        messages: list[dict[str, str]],
        model_id: str,
        *,
        temperature: float | None = None,
        max_tokens: int | None = None,
    ) -> str:
        """Send a prompt to the Ollama API."""
        client = self._get_client()

        model_name = self.strip_model_prefix(model_id)

        payload: dict[str, Any] = {
            "model": model_name,
            "messages": messages,
            "response_format": {"type": "json_object"},
            "stream": False,
        }

        if temperature is not None:
            payload["temperature"] = temperature
        if max_tokens is not None:
            payload["max_tokens"] = max_tokens

        url = f"{self._base_url()}/v1/chat/completions"

        try:
            resp = await client.post(url, json=payload)
            resp.raise_for_status()
            data: dict[str, Any] = resp.json()
            return str(data["choices"][0]["message"]["content"])
        except Exception as exc:
            raise EamlProviderError(
                provider="ollama",
                message=str(exc),
            ) from exc

    async def close(self) -> None:
        """Close the httpx client to release connections."""
        if self._client is not None:
            await self._client.aclose()
            self._client = None
