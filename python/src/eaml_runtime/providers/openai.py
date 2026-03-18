"""OpenAI provider adapter."""

from __future__ import annotations

import os
from typing import Any

from eaml_runtime.errors import EamlConfigError, EamlProviderError
from eaml_runtime.providers import Provider


class OpenAIProvider(Provider):
    """Provider adapter for OpenAI GPT models."""

    _client: Any = None

    def _get_client(self) -> Any:
        """Get or create the OpenAI async client."""
        if self._client is None:
            try:
                import openai  # noqa: F811
            except ImportError:
                raise EamlConfigError("Install openai SDK: pip install openai") from None

            api_key = os.environ.get("OPENAI_API_KEY")
            if not api_key:
                raise EamlConfigError(
                    "OPENAI_API_KEY environment variable is not set. "
                    "Set it with: export OPENAI_API_KEY=your-key"
                )
            self._client = openai.AsyncOpenAI(api_key=api_key)
        return self._client

    async def send_prompt(
        self,
        messages: list[dict[str, str]],
        model_id: str,
        *,
        temperature: float | None = None,
        max_tokens: int | None = None,
    ) -> str:
        """Send a prompt to the OpenAI API."""
        client = self._get_client()

        model_name = self.strip_model_prefix(model_id)

        kwargs: dict[str, Any] = {
            "model": model_name,
            "messages": messages,
            "response_format": {"type": "json_object"},
        }

        if temperature is not None:
            kwargs["temperature"] = temperature
        if max_tokens is not None:
            kwargs["max_tokens"] = max_tokens

        try:
            response = await client.chat.completions.create(**kwargs)
            return str(response.choices[0].message.content or "")
        except EamlConfigError:
            raise
        except Exception as exc:
            raise EamlProviderError(
                provider="openai",
                message=str(exc),
            ) from exc
