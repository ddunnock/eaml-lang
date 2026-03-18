"""Anthropic provider adapter."""

from __future__ import annotations

import os
from typing import Any

from eaml_runtime.errors import EamlConfigError, EamlProviderError
from eaml_runtime.providers import Provider


class AnthropicProvider(Provider):
    """Provider adapter for Anthropic Claude models."""

    _client: Any = None

    def _get_client(self) -> Any:
        """Get or create the Anthropic async client."""
        if self._client is None:
            try:
                import anthropic  # noqa: F811
            except ImportError:
                raise EamlConfigError("Install anthropic SDK: pip install anthropic") from None

            api_key = os.environ.get("ANTHROPIC_API_KEY")
            if not api_key:
                raise EamlConfigError(
                    "ANTHROPIC_API_KEY environment variable is not set. "
                    "Set it with: export ANTHROPIC_API_KEY=your-key"
                )
            self._client = anthropic.AsyncAnthropic(api_key=api_key)
        return self._client

    async def send_prompt(
        self,
        messages: list[dict[str, str]],
        model_id: str,
        *,
        temperature: float | None = None,
        max_tokens: int | None = None,
    ) -> str:
        """Send a prompt to the Anthropic API."""
        client = self._get_client()

        # Extract system messages (Anthropic uses top-level system param)
        system_parts = [m["content"] for m in messages if m["role"] == "system"]
        non_system = [m for m in messages if m["role"] != "system"]

        model_name = self.strip_model_prefix(model_id)

        kwargs: dict[str, Any] = {
            "model": model_name,
            "messages": non_system,
            "max_tokens": max_tokens or 4096,
        }

        json_instruction = "Respond with valid JSON only."
        system_parts.append(json_instruction)
        kwargs["system"] = "\n\n".join(system_parts)

        if temperature is not None:
            kwargs["temperature"] = temperature

        try:
            response = await client.messages.create(**kwargs)
            return str(response.content[0].text)
        except EamlConfigError:
            raise
        except Exception as exc:
            raise EamlProviderError(
                provider="anthropic",
                message=str(exc),
            ) from exc
