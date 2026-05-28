"""Custom exceptions for jab-rpa.

Maps gRPC status codes from the JAB server to descriptive Python
exceptions, so callers never have to handle bare ``grpc.RpcError``.

- **WindowNotFound** — No Java window matches the given title pattern
- **ServerStoppedError** — The `jab-rpa-server.exe` process exits prematurely
- **JabError** — base exception for all jab-rpa server-side errors
- **JabInvalidArgumentError** — invalid argument
- **JabNoWindowError** — no window selected
- **JabElementNotFoundError** — element not found
- **JabTimeoutError** — operation timed out
- **JabInternalError** — internal server error
"""

import grpc
from typing import cast


class ServerStoppedError(Exception):
    """Raised when the JAB gRPC server process exits before it starts listening."""


class WindowNotFound(Exception):
    """Raised when no Java window matching the given title is found within the timeout."""


class JabError(Exception):
    """Base exception for all jab-rpa errors."""


class JabInvalidArgumentError(JabError):
    """Raised when an invalid argument is sent to the server.

    Corresponds to gRPC ``INVALID_ARGUMENT`` — e.g. a malformed
    selector string or an HWND that does not belong to a Java window.
    """


class JabNoWindowError(JabError):
    """Raised when no window has been selected.

    Corresponds to gRPC ``FAILED_PRECONDITION`` — ``SelectWindow``
    must be called before most operations.
    """


class JabElementNotFoundError(JabError):
    """Raised when no element matches the given selector.

    Corresponds to gRPC ``NOT_FOUND``.
    """


class JabTimeoutError(JabError, TimeoutError):
    """Raised when an operation exceeds the deadline.

    Corresponds to gRPC ``DEADLINE_EXCEEDED`` — the server waited
    for the full timeout without finding a match.
    """


class JabInternalError(JabError):
    """Raised when the server encounters an internal failure.

    Corresponds to gRPC ``INTERNAL`` — e.g. a JAB bridge call that
    failed unexpectedly.
    """


_CODE_MAP: dict[grpc.StatusCode, type[JabError]] = {
    grpc.StatusCode.INVALID_ARGUMENT: JabInvalidArgumentError,
    grpc.StatusCode.FAILED_PRECONDITION: JabNoWindowError,
    grpc.StatusCode.NOT_FOUND: JabElementNotFoundError,
    grpc.StatusCode.DEADLINE_EXCEEDED: JabTimeoutError,
    grpc.StatusCode.INTERNAL: JabInternalError,
}


def raise_rpc_error(err: grpc.RpcError) -> None:
    """Re-raise a gRPC error as the corresponding ``JabError`` subclass.

    Args:
        err: The caught ``grpc.RpcError``.

    Raises:
        JabInvalidArgumentError
        JabNoWindowError
        JabElementNotFoundError
        JabTimeoutError
        JabInternalError
        JabError: For unrecognized status codes.
    """
    call = cast(grpc.Call, err)
    code: grpc.StatusCode = call.code()
    details = call.details() or str(err)
    exc_class = _CODE_MAP.get(code)
    if exc_class is not None:
        raise exc_class(details) from err
    raise JabError(f"Unexpected gRPC error ({code.name}): {details}") from err


__all__ = [
    "ServerStoppedError",
    "WindowNotFound",
    "JabError",
    "JabInvalidArgumentError",
    "JabNoWindowError",
    "JabElementNotFoundError",
    "JabTimeoutError",
    "JabInternalError",
    "raise_rpc_error",
]
