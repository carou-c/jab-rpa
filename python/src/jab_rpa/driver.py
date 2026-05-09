import re
from typing import Self
import time
from pathlib import Path

from win32gui import ShowWindow, SetForegroundWindow
from win32con import SW_SHOWMAXIMIZED

from .server import (
    JabRpaServer,
    _SERVER_PATH,
    _WAIT_FOR_SERVER_TIMEOUT,
    _INIT_SERVER_STEP,
)
from .client import JabRpaClient
from .types import VersionInfo, WindowInfo, Element, Locator

_WAIT_FOR_WINDOW_TIMEOUT: int = 60
_WAIT_FOR_WINDOW_STEP: int = 5


class WindowNotFound(Exception):
    """Raised when window with specified title is not found"""


class JabDriver:
    def __init__(
        self,
        window_title: str | re.Pattern[str],
        *,
        server_path: Path = _SERVER_PATH,
        server_timeout: int = _WAIT_FOR_SERVER_TIMEOUT,
        server_step: int = _INIT_SERVER_STEP,
        window_timeout: int = _WAIT_FOR_WINDOW_TIMEOUT,
        window_step: int = _WAIT_FOR_WINDOW_STEP,
    ) -> None:
        self.__window_title: str | re.Pattern[str] = window_title
        self.__server_path: Path = server_path
        self.__server_timeout: int = server_timeout
        self.__window_timeout: int = window_timeout
        self.__server_step: int = server_step
        self.__window_step: int = window_step

    def start(self) -> None:
        self.__server: JabRpaServer = JabRpaServer(
            server_path=self.__server_path,
            server_timeout=self.__server_timeout,
            step=self.__server_step,
        )
        self.__client: JabRpaClient = JabRpaClient()

        wait_start = time.monotonic()
        while time.monotonic() - wait_start <= self.__window_timeout:
            windows = [
                window
                for window in self.__client.list_java_windows()
                if re.search(self.__window_title, window.title)
            ]
            if windows:
                window_info = windows[0]
                break
            time.sleep(self.__window_step)
        else:
            raise WindowNotFound(
                f"Java window with title matching {self.__window_title!r} not found within timeout {self.__window_timeout!r}."
            )

        SetForegroundWindow(window_info.hwnd)
        ShowWindow(window_info.hwnd, SW_SHOWMAXIMIZED)
        self.__client.select_window(window_info)

    def __enter__(self) -> Self:
        self.start()
        return self

    def stop(self) -> None:
        self.__client.stop()
        self.__server.stop()

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.stop()

    def get_children(self, element: Element) -> list[Element]:
        children: list[Element] = []
        for handle in element._element.children_handles:
            child = self.__client.get_element_from_handle(handle)
            if child is not None:
                children.append(Element(child, self))
        return children

    def get_parent(self, element: Element) -> Element | None:
        handle = element._element.parent_handle
        parent = None
        if handle is not None:
            parent = self.__client.get_element_from_handle(handle)
        if parent is not None:
            return Element(parent, self)

    def accessible_click(self, element: Element) -> None:
        self.__client.click_element(element._element)

    def matching(self, locator: Locator) -> list[Element]:
        return [
            Element(el, self) for el in self.__client.find_elements(locator._locator)
        ]

    def list_java_windows(self) -> list[WindowInfo]:
        return self.__client.list_java_windows()

    def select_window(self, window_info: WindowInfo) -> None:
        return self.__client.select_window(window_info)

    def refresh_tree(self) -> None:
        return self.__client.refresh_tree()

    def get_version_info(self) -> VersionInfo | None:
        return self.__client.get_version_info()

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
