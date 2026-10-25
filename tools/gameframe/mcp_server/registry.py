"""Tool schemas and dispatch for the GameFrame MCP server.

Usage: `build_registry()` gathers capability modules; `Registry.call` invokes one tool.
Inputs: Tool definitions, JSON-compatible argument objects and a request context.
Outputs: MCP tool definitions and normalized call values; no process or protocol I/O.
"""
from dataclasses import dataclass


class ToolCallError(Exception):
    """A concise error caused by invalid input or an unavailable tool."""


@dataclass(frozen=True)
class ToolOutput:
    """A JSON-compatible value and the MCP error flag for one successful dispatch."""
    value: object
    is_error: bool = False


@dataclass(frozen=True)
class Tool:
    name: str
    description: str
    input_schema: dict
    handler: object

    def as_mcp(self):
        return {'name': self.name, 'description': self.description,
                'inputSchema': self.input_schema}


def schema(properties=None, required=()):
    """Return a small JSON Schema object for an MCP tool."""
    return {'type': 'object', 'properties': properties or {}, 'required': list(required)}


class Registry:
    def __init__(self, tools):
        self._tools = {tool.name: tool for tool in tools}

    def definitions(self):
        return [tool.as_mcp() for tool in self._tools.values()]

    def call(self, name, arguments, context):
        tool = self._tools.get(name)
        if tool is None:
            return f'unknown GameFrame MCP tool {name!r}', True
        if not isinstance(arguments, dict):
            return 'tool arguments must be a JSON object', True
        try:
            value = tool.handler(arguments, context)
            if isinstance(value, ToolOutput):
                return value.value, value.is_error
            return value, False
        except ToolCallError as err:
            return str(err), True
        except InterruptedError as err:
            return str(err), True
        except Exception as err:  # report a tool failure without stopping the server
            return f'{type(err).__name__}: {err}', True


def build_registry():
    """Load the independent tool groups in a stable order."""
    from mcp_server.toolsets import discovery, knowledge, runner

    return Registry([*knowledge.tools(), *discovery.tools(), *runner.tools()])
