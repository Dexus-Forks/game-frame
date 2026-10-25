"""Compatibility entry point for GameFrame's modular MCP server.

Usage: the CLI runs this module with `gameframe serve`.
Inputs and outputs: newline-delimited JSON-RPC on stdin and stdout.
This module only delegates; MCP protocol and tool groups live in `mcp_server/`.
"""
from mcp_server.server import serve

__all__ = ['serve']
