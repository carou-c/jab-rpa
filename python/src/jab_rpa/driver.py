import re
from typing import Self
import time
import threading
import queue
import subprocess
from pathlib import Path
from importlib.resources import files

from .client import JabRpaClient
from .types import VersionInfo, WindowInfo, Locator

_SERVER_PATH = Path(str(files("jab_rpa").joinpath("bin/jab-rpa-server.exe")))
_SERVER_LISTENING = "JAB gRPC Server listening on 127.0.0.1:50051"

_WAIT_FOR_SERVER_TIMEOUT: int = 30
_WAIT_FOR_WINDOW_TIMEOUT: int = 60
_INIT_DRIVER_STEP: int = 5


class ServerStoppedError(Exception):
    """Raised when JAB gRPC server stops before listening"""


class WindowNotFound(Exception):
    """Raised when window with specified title is not found"""


class JabDriver:
    def __init__(
        self,
        window_title: str | re.Pattern[str] | None = None,
        *,
        server_path: Path = _SERVER_PATH,
        server_timeout: int = _WAIT_FOR_SERVER_TIMEOUT,
        window_timeout: int = _WAIT_FOR_WINDOW_TIMEOUT,
        step: int = _INIT_DRIVER_STEP,
    ) -> None:
        self.__window_title: str | re.Pattern[str] | None = window_title
        self.__server_path: Path = server_path
        self.__server_timeout: int = server_timeout
        self.__window_timeout: int = window_timeout
        self.__step: int = step

    def __enter__(self) -> Self:
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
        self._client: JabRpaClient = JabRpaClient()

        if self.__window_title is not None:
            wait_start = time.monotonic()
            while time.monotonic() - wait_start <= self.__window_timeout:
                windows = [
                    window
                    for window in self._client.list_java_windows()
                    if re.search(self.__window_title, window.title)
                ]
                if windows:
                    window_info = windows[0]
                    break
            else:
                raise WindowNotFound(
                    f"Java window with title matching {self.__window_title} not found."
                )
            self._client.select_window(window_info)

        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self._client.__exit__(exc_type, exc_val, exc_tb)
        self.__server_proc.terminate()

    def list_java_windows(self) -> list[WindowInfo]:
        return self._client.list_java_windows()

    def select_window(self, window_info: WindowInfo) -> None:
        return self._client.select_window(window_info)

    def refresh_tree(self) -> None:
        return self._client.refresh_tree()

    def get_version_info(self) -> VersionInfo | None:
        return self._client.get_version_info()

    def locator(
        self,
        *,
        name: str | None = None,
        role: str | None = None,
        description: str | None = None,
        text: str | None = None,
        has_state: list[str] | None = None,
        not_has_state: list[str] | None = None,
        index_in_parent: int | None = None,
        has_children: list["Locator"] | None = None,
        has_descendants: list["Locator"] | None = None,
        name_regex: bool = True,
        role_regex: bool = True,
        description_regex: bool = True,
        text_regex: bool = True,
    ) -> Locator:
        return Locator(
            self,
            name=name,
            role=role,
            description=description,
            text=text,
            has_state=has_state,
            not_has_state=not_has_state,
            index_in_parent=index_in_parent,
            has_children=has_children,
            has_descendants=has_descendants,
            name_regex=name_regex,
            role_regex=role_regex,
            description_regex=description_regex,
            text_regex=text_regex,
        )
