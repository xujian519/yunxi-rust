from __future__ import annotations

from dataclasses import dataclass, field


@dataclass(frozen=True)
class Subsystem:
    """Describes a subsystem within the project, including its path and file count."""
    name: str
    path: str
    file_count: int
    notes: str


@dataclass(frozen=True)
class PortingModule:
    """Represents a module being ported, with its name, responsibility, and source origin."""
    name: str
    responsibility: str
    source_hint: str
    status: str = 'planned'


@dataclass(frozen=True)
class PermissionDenial:
    """Records a tool permission denial with the tool name and reason."""
    tool_name: str
    reason: str


@dataclass(frozen=True)
class UsageSummary:
    """Tracks approximate token usage across conversation turns."""
    input_tokens: int = 0
    output_tokens: int = 0

    def add_turn(self, prompt: str, output: str) -> 'UsageSummary':
        """Return a new summary with token estimates added for the given turn."""
        return UsageSummary(
            input_tokens=self.input_tokens + len(prompt.split()),
            output_tokens=self.output_tokens + len(output.split()),
        )


@dataclass
class PortingBacklog:
    """A titled collection of porting modules representing a porting surface."""
    title: str
    modules: list[PortingModule] = field(default_factory=list)

    def summary_lines(self) -> list[str]:
        """Render each module as a bullet line with name, status, and source hint."""
        return [
            f'- {module.name} [{module.status}] — {module.responsibility} (from {module.source_hint})'
            for module in self.modules
        ]
