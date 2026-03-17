"""Core orchestration: validate_or_retry and execute_prompt."""

from __future__ import annotations

import json
import time
from typing import Any, Literal, get_args, get_origin

from pydantic import ValidationError

from eaml_runtime.errors import EamlError, EamlProviderError, EamlValidationError
from eaml_runtime.events import CallEndEvent, CallStartEvent, ValidationFailureEvent
from eaml_runtime.providers import Provider, get_provider
from eaml_runtime.telemetry import _fire

_PRIMITIVE_TYPES: tuple[type, ...] = (str, int, float, bool)


def _is_primitive(return_type: Any) -> bool:
    """Check if return_type is a primitive type or a Literal."""
    if return_type in _PRIMITIVE_TYPES:
        return True
    return get_origin(return_type) is Literal


def _validate_primitive(raw: str, return_type: Any) -> Any:
    """Parse and validate a primitive or Literal value from raw JSON string."""
    value = json.loads(raw)

    if get_origin(return_type) is Literal:
        allowed = get_args(return_type)
        if value not in allowed:
            msg = f"Value {value!r} is not one of the allowed literals: {allowed}"
            raise ValueError(msg)
        return value

    if not isinstance(value, return_type):
        # json.loads returns int for 42, float for 3.14, bool for true/false, str for "..."
        # But bool is subclass of int, so check bool first
        if return_type is float and isinstance(value, int):
            return float(value)
        msg = f"Expected {return_type.__name__}, got {type(value).__name__}"
        raise ValueError(msg)

    return value


async def validate_or_retry(
    provider: Provider,
    messages: list[dict[str, str]],
    model_id: str,
    return_type: Any,
    *,
    max_retries: int = 3,
    provider_name: str = "",
    **kwargs: Any,
) -> Any:
    """Call LLM and validate response, retrying on validation failures.

    On validation failure, appends error feedback to messages and retries.
    Provider errors bubble up immediately without retry.
    After max_retries exhausted, raises EamlValidationError.
    """
    errors: list[str] = []
    raw = ""

    for attempt in range(1, max_retries + 1):
        raw = await provider.send_prompt(messages, model_id, **kwargs)

        try:
            if _is_primitive(return_type):
                return _validate_primitive(raw, return_type)
            else:
                return return_type.model_validate_json(raw)
        except (ValidationError, ValueError, json.JSONDecodeError) as exc:
            error_msg = str(exc)
            errors.append(error_msg)

            _fire(
                "on_validation_failure",
                ValidationFailureEvent(
                    provider=provider_name,
                    model_id=model_id,
                    attempt=attempt,
                    error=error_msg,
                ),
            )

            # Append error feedback for retry
            messages.append({
                "role": "user",
                "content": (
                    f"Your response was not valid. Error: {error_msg}\n"
                    "Please try again with a valid JSON response."
                ),
            })

    raise EamlValidationError(
        model_id=model_id,
        attempts=max_retries,
        last_response=raw,
        errors=errors,
    )


async def execute_prompt(
    *,
    model: dict[str, Any],
    messages: list[dict[str, str]],
    return_type: Any,
    temperature: float | None = None,
    max_tokens: int | None = None,
    max_retries: int = 3,
) -> Any:
    """Top-level entry point for EAML-generated code.

    Dispatches to the correct provider, validates the response,
    retries on validation failure, and fires telemetry events.
    """
    provider_name: str = model["provider"]
    model_id: str = model["model_id"]
    provider = get_provider(provider_name)

    _fire("on_call_start", CallStartEvent(provider=provider_name, model_id=model_id))
    start = time.time()

    kwargs: dict[str, Any] = {}
    if temperature is not None:
        kwargs["temperature"] = temperature
    if max_tokens is not None:
        kwargs["max_tokens"] = max_tokens

    try:
        result = await validate_or_retry(
            provider,
            messages,
            model_id,
            return_type,
            max_retries=max_retries,
            provider_name=provider_name,
            **kwargs,
        )
        duration = time.time() - start
        _fire(
            "on_call_end",
            CallEndEvent(
                provider=provider_name,
                model_id=model_id,
                duration=duration,
            ),
        )
        return result
    except EamlError:
        raise
    except Exception as exc:
        raise EamlProviderError(
            provider=provider_name,
            message=str(exc),
        ) from exc
