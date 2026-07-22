from typing import Self, Literal
import time
import threading
import queue
import subprocess
from pathlib import Path
from importlib.resources import files

from .errors import ServerStoppedError


_BIN_PATH = files("jab_rpa").joinpath("bin")
_SERVER_LISTENING = "JAB gRPC Server listening on "  # 127.0.0.1:50051"

_WAIT_FOR_SERVER_TIMEOUT: int = 30
_INIT_SERVER_STEP: int = 1


class JabRpaServer:
    """Manages the lifecycle of the ``jab-rpa-server.exe`` subprocess.

    Spawns the 32-bit Rust gRPC server, waits for it to report readiness,
    and provides clean shutdown.

    Can be used as a context manager:

        with JabRpaServer() as server:
            ...
    """

    def __init__(
        self,
        *,
        java_bitness: Literal["32 bit", "64 bit"] = "32 bit",
        java_version: Literal["8", "11", "17", "21", "25"] = "8",
        server_address: str = "127.0.0.1",
        server_port: str = "50051",
        server_timeout: int = _WAIT_FOR_SERVER_TIMEOUT,
        step: int = _INIT_SERVER_STEP,
        print_stdout: bool = False,
        print_stderr: bool = True,
    ) -> None:
        """Configure the server process settings.

        Args:
            java_bitness: ``"32 bit"`` (default) or ``"64 bit"``. Controls which
                pre-built server binary is used.
            java_version: ``"8"`` (default), ``"11"``, ``"17"``, ``"21"``, or
                ``"25"``. Java version the target application runs on. Only Java 8
                has been proven to work reliably in production.
            server_address: Server bind address (default ``"127.0.0.1"``).
            server_port: Server bind port (default ``"50051"``).
            server_timeout: Maximum seconds to wait for the server to start.
            step: Seconds between readiness checks.
            print_stdout: If ``True``, forward server stdout to the console.
            print_stderr: If ``True`` (default), forward server stderr to the console.
        """
        target_name = (
            "i686-pc-windows-gnu"
            if java_bitness == "32 bit"
            else "x86_64-pc-windows-gnu"
        )

        self._server_path: Path = Path(
            str(_BIN_PATH.joinpath(f"java-{java_version}").joinpath(target_name))
        )
        self._server_address: str = server_address
        self._server_port: str = server_port
        self._server_timeout: int = server_timeout
        self._step: int = step
        self._print_stdout: bool = print_stdout
        self._print_stderr: bool = print_stderr

    def start(self) -> None:
        """Launch the server subprocess and wait for it to become ready.

        Reads stdout in a background daemon thread and watches for the
        server's "listening" message.

        Raises:
            ServerStoppedError: If the process exits before reporting readiness.
            TimeoutError: If the server does not become ready within the timeout.
        """
        server_proc = subprocess.Popen(
            [
                self._server_path,
                "--address",
                self._server_address,
                "--port",
                self._server_port,
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        self._server_proc = server_proc

        q: queue.Queue[str] = queue.Queue()

        def _reader():
            if server_proc.stdout is not None:
                for line in iter(server_proc.stdout.readline, ""):
                    q.put(line)
                    if self._print_stdout:
                        print(line)

        t = threading.Thread(target=_reader, daemon=True)
        t.start()

        if self._print_stderr:

            def _reader_err():
                if server_proc.stderr is not None:
                    for line in iter(server_proc.stderr.readline, ""):
                        print(line)

            t_err = threading.Thread(target=_reader_err, daemon=True)
            t_err.start()

        stdout = ""
        wait_start = time.monotonic()
        while time.monotonic() - wait_start <= self._server_timeout:
            while True:
                try:
                    stdout += q.get_nowait()
                except queue.Empty:
                    break
            if _SERVER_LISTENING in stdout:
                break

            if (status := server_proc.poll()) is not None:
                raise ServerStoppedError(
                    "JAB gRPC server process stopped before listening.\n"
                    f"Exit code: {status}\n"
                    f"stderr: {server_proc.stderr.read() if server_proc.stderr is not None else None}\n"
                    f"stdout: {server_proc.stdout.read() if server_proc.stdout is not None else None}\n"
                )

            time.sleep(self._step)
        else:
            raise TimeoutError(
                f"Timeout ({self._server_timeout} seconds) passed while waiting"
                " for JAB gRPC server to start"
            )

    def __enter__(self) -> Self:
        self.start()
        return self

    def stop(self) -> None:
        """Terminate the server subprocess."""
        self._server_proc.terminate()
        self._server_proc.wait()

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.stop()
