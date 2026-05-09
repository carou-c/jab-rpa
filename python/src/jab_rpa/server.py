from typing import Self
import time
import threading
import queue
import subprocess
from pathlib import Path
from importlib.resources import files

_SERVER_PATH = Path(str(files("jab_rpa").joinpath("bin/jab-rpa-server.exe")))
_SERVER_LISTENING = "JAB gRPC Server listening on 127.0.0.1:50051"

_WAIT_FOR_SERVER_TIMEOUT: int = 30
_INIT_SERVER_STEP: int = 5


class ServerStoppedError(Exception):
    """Raised when JAB gRPC server stops before listening"""


class JabRpaServer:
    def __init__(
        self,
        *,
        server_path: Path = _SERVER_PATH,
        server_timeout: int = _WAIT_FOR_SERVER_TIMEOUT,
        step: int = _INIT_SERVER_STEP,
    ) -> None:
        self.__server_path: Path = server_path
        self.__server_timeout: int = server_timeout
        self.__step: int = step

    def start(self) -> None:
        server_proc = subprocess.Popen(
            [self.__server_path],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
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
        while time.monotonic() - wait_start <= self.__server_timeout:
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
                    f"Exit code: {status}"
                    f"stderr: {server_proc.stderr.read() if server_proc.stderr is not None else None}\n"
                    f"stdout: {server_proc.stdout.read() if server_proc.stdout is not None else None}\n"
                )

            time.sleep(self.__step)

        self.__server_proc = server_proc

    def __enter__(self) -> Self:
        self.start()
        return self

    def stop(self) -> None:
        self.__server_proc.terminate()

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.stop()
