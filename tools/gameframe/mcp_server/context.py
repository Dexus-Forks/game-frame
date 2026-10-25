"""Request-scoped progress and cancellation for MCP tool handlers.

Usage: the server creates one context per `tools/call` request and passes it to a handler.
Inputs: request ID, optional progress token and a function that writes JSON-RPC notifications.
Outputs: progress notifications and a cancellation event; this module runs no tools itself.
"""
import threading


class RequestContext:
    def __init__(self, request_id, progress_token=None, notify=None):
        self.request_id = request_id
        self.progress_token = progress_token
        self._notify = notify or (lambda message: None)
        self.cancelled = threading.Event()
        self._last_progress = None

    def report_progress(self, progress, total=None, message=None):
        """Send a progress notification when the client supplied a token."""
        if self.progress_token is None or self.cancelled.is_set():
            return
        if self._last_progress is not None and progress <= self._last_progress:
            return
        self._last_progress = progress
        params = {'progressToken': self.progress_token, 'progress': progress}
        if total is not None:
            params['total'] = total
        if message:
            params['message'] = message
        self._notify({'jsonrpc': '2.0', 'method': 'notifications/progress', 'params': params})

    def check_cancelled(self):
        """Raise InterruptedError when the client cancelled this request."""
        if self.cancelled.is_set():
            raise InterruptedError('MCP request was cancelled')
