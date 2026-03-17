"""Agent base class and ToolMetadata dataclass."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Callable


@dataclass
class ToolMetadata:
    """Metadata for a tool that an agent can invoke."""

    name: str
    description: str
    parameters: list[dict[str, str]]
    return_type: str
    function: Callable[..., Any]


class Agent:
    """Base class for EAML-generated agents.

    Generated code creates subclasses that override class attributes.
    The orchestration loop (Agent.run) is deferred to post-MVP.
    """

    model: dict[str, Any] = {}
    tools: list[Any] = []
    system_prompt: str = ""
    max_turns: int = 10
    on_error: str = "fail"
    on_error_retries: int = 0
