"""Telemetry event dataclasses."""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime


@dataclass
class CallStartEvent:
    """Fired when an LLM call begins."""

    provider: str
    model_id: str
    timestamp: datetime = field(default_factory=datetime.now)


@dataclass
class CallEndEvent:
    """Fired when an LLM call completes."""

    provider: str
    model_id: str
    duration: float
    token_usage: dict[str, int] | None = None
    timestamp: datetime = field(default_factory=datetime.now)


@dataclass
class ValidationFailureEvent:
    """Fired when Pydantic validation fails on LLM output."""

    provider: str
    model_id: str
    attempt: int
    error: str
    timestamp: datetime = field(default_factory=datetime.now)


@dataclass
class ToolCallEvent:
    """Fired when an agent invokes a tool."""

    tool_name: str
    agent_name: str
    timestamp: datetime = field(default_factory=datetime.now)
