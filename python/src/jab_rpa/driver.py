from typing import Self
import time
import threading
import queue
import subprocess
from pathlib import Path
from importlib.resources import files

from .client import JabRpaClient
from .locator import Locator


_WAIT_FOR_SERVER_TIMEOUT: int = 30
_WAIT_FOR_SERVER_STEP: int = 5
_SERVER_LISTENING = "JAB gRPC Server listening on 127.0.0.1:50051"
_SERVER_PATH = Path(str(files("jab_rpa").joinpath("bin/jab-rpa-server.exe")))


class ServerStoppedError(Exception):
    """Raised when JAB gRPC server stops before listening"""


class LazyElement:
    def __init__(self, locator: Locator, driver: "JabDriver"):
        self._locator: Locator = locator
        self._driver: "JabDriver" = driver


class JabDriver(JabRpaClient):
    def __init__(
        self,
        server_path: Path = _SERVER_PATH,
        timeout: int = _WAIT_FOR_SERVER_TIMEOUT,
        step: int = _WAIT_FOR_SERVER_STEP,
    ) -> None:
        server_proc = subprocess.Popen(
            [server_path], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True
        )

        q: queue.Queue[str] = queue.Queue()

        def _reader():
            if server_proc.stdout is not None:
                for line in iter(server_proc.stdout.readline, ""):
                    q.put(line)

        t = threading.Thread(target=_reader, daemon=True)
        t.start()

        stdout = ""
        wait_start = time.monotonic()
        while time.monotonic() - wait_start <= timeout:
            while True:
                try:
                    stdout += q.get_nowait()
                except queue.Empty:
                    break
            print(stdout)
            if _SERVER_LISTENING in stdout:
                break

            if (status := server_proc.poll()) is not None:
                raise ServerStoppedError(
                    "JAB gRPC server process stopped before listening.\n"
                    f"Exit code: {status}"
                    f"stderr: {server_proc.stderr.read() if server_proc.stderr is not None else None}\n"
                    f"stdout: {server_proc.stdout.read() if server_proc.stdout is not None else None}\n"
                )

            time.sleep(step)

        self._server_proc = server_proc
        super().__init__()

    def __enter__(self) -> Self:
        self = super().__enter__()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        super().__exit__(exc_type, exc_val, exc_tb)
        self._server_proc.terminate()


