from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ListJavaWindowsRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class WindowInfo(_message.Message):
    __slots__ = ("vm_id", "hwnd", "title", "role")
    VM_ID_FIELD_NUMBER: _ClassVar[int]
    HWND_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    ROLE_FIELD_NUMBER: _ClassVar[int]
    vm_id: int
    hwnd: int
    title: str
    role: str
    def __init__(self, vm_id: _Optional[int] = ..., hwnd: _Optional[int] = ..., title: _Optional[str] = ..., role: _Optional[str] = ...) -> None: ...

class ListJavaWindowsResponse(_message.Message):
    __slots__ = ("windows",)
    WINDOWS_FIELD_NUMBER: _ClassVar[int]
    windows: _containers.RepeatedCompositeFieldContainer[WindowInfo]
    def __init__(self, windows: _Optional[_Iterable[_Union[WindowInfo, _Mapping]]] = ...) -> None: ...

class SelectWindowByTitleRequest(_message.Message):
    __slots__ = ("title", "partial_match")
    TITLE_FIELD_NUMBER: _ClassVar[int]
    PARTIAL_MATCH_FIELD_NUMBER: _ClassVar[int]
    title: str
    partial_match: bool
    def __init__(self, title: _Optional[str] = ..., partial_match: bool = ...) -> None: ...

class SelectWindowByTitleResponse(_message.Message):
    __slots__ = ("success", "error_message")
    SUCCESS_FIELD_NUMBER: _ClassVar[int]
    ERROR_MESSAGE_FIELD_NUMBER: _ClassVar[int]
    success: bool
    error_message: str
    def __init__(self, success: bool = ..., error_message: _Optional[str] = ...) -> None: ...

class SelectWindowByPidRequest(_message.Message):
    __slots__ = ("pid",)
    PID_FIELD_NUMBER: _ClassVar[int]
    pid: int
    def __init__(self, pid: _Optional[int] = ...) -> None: ...

class SelectWindowByPidResponse(_message.Message):
    __slots__ = ("success", "error_message")
    SUCCESS_FIELD_NUMBER: _ClassVar[int]
    ERROR_MESSAGE_FIELD_NUMBER: _ClassVar[int]
    success: bool
    error_message: str
    def __init__(self, success: bool = ..., error_message: _Optional[str] = ...) -> None: ...

class GetElementsRequest(_message.Message):
    __slots__ = ("locator",)
    LOCATOR_FIELD_NUMBER: _ClassVar[int]
    locator: str
    def __init__(self, locator: _Optional[str] = ...) -> None: ...

class Element(_message.Message):
    __slots__ = ("handle", "name", "role", "description", "x", "y", "width", "height", "accessible_action", "accessible_text", "accessible_selection")
    HANDLE_FIELD_NUMBER: _ClassVar[int]
    NAME_FIELD_NUMBER: _ClassVar[int]
    ROLE_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    X_FIELD_NUMBER: _ClassVar[int]
    Y_FIELD_NUMBER: _ClassVar[int]
    WIDTH_FIELD_NUMBER: _ClassVar[int]
    HEIGHT_FIELD_NUMBER: _ClassVar[int]
    ACCESSIBLE_ACTION_FIELD_NUMBER: _ClassVar[int]
    ACCESSIBLE_TEXT_FIELD_NUMBER: _ClassVar[int]
    ACCESSIBLE_SELECTION_FIELD_NUMBER: _ClassVar[int]
    handle: int
    name: str
    role: str
    description: str
    x: int
    y: int
    width: int
    height: int
    accessible_action: bool
    accessible_text: bool
    accessible_selection: bool
    def __init__(self, handle: _Optional[int] = ..., name: _Optional[str] = ..., role: _Optional[str] = ..., description: _Optional[str] = ..., x: _Optional[int] = ..., y: _Optional[int] = ..., width: _Optional[int] = ..., height: _Optional[int] = ..., accessible_action: bool = ..., accessible_text: bool = ..., accessible_selection: bool = ...) -> None: ...

class GetElementsResponse(_message.Message):
    __slots__ = ("elements", "error_message")
    ELEMENTS_FIELD_NUMBER: _ClassVar[int]
    ERROR_MESSAGE_FIELD_NUMBER: _ClassVar[int]
    elements: _containers.RepeatedCompositeFieldContainer[Element]
    error_message: str
    def __init__(self, elements: _Optional[_Iterable[_Union[Element, _Mapping]]] = ..., error_message: _Optional[str] = ...) -> None: ...

class ClickElementRequest(_message.Message):
    __slots__ = ("handle",)
    HANDLE_FIELD_NUMBER: _ClassVar[int]
    handle: int
    def __init__(self, handle: _Optional[int] = ...) -> None: ...

class ClickElementResponse(_message.Message):
    __slots__ = ("success", "error_message")
    SUCCESS_FIELD_NUMBER: _ClassVar[int]
    ERROR_MESSAGE_FIELD_NUMBER: _ClassVar[int]
    success: bool
    error_message: str
    def __init__(self, success: bool = ..., error_message: _Optional[str] = ...) -> None: ...

class TypeTextRequest(_message.Message):
    __slots__ = ("handle", "text")
    HANDLE_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    handle: int
    text: str
    def __init__(self, handle: _Optional[int] = ..., text: _Optional[str] = ...) -> None: ...

class TypeTextResponse(_message.Message):
    __slots__ = ("success", "error_message")
    SUCCESS_FIELD_NUMBER: _ClassVar[int]
    ERROR_MESSAGE_FIELD_NUMBER: _ClassVar[int]
    success: bool
    error_message: str
    def __init__(self, success: bool = ..., error_message: _Optional[str] = ...) -> None: ...

class ReadTableRequest(_message.Message):
    __slots__ = ("locator",)
    LOCATOR_FIELD_NUMBER: _ClassVar[int]
    locator: str
    def __init__(self, locator: _Optional[str] = ...) -> None: ...

class TableCell(_message.Message):
    __slots__ = ("value", "row", "column")
    VALUE_FIELD_NUMBER: _ClassVar[int]
    ROW_FIELD_NUMBER: _ClassVar[int]
    COLUMN_FIELD_NUMBER: _ClassVar[int]
    value: str
    row: int
    column: int
    def __init__(self, value: _Optional[str] = ..., row: _Optional[int] = ..., column: _Optional[int] = ...) -> None: ...

class TableRow(_message.Message):
    __slots__ = ("cells",)
    CELLS_FIELD_NUMBER: _ClassVar[int]
    cells: _containers.RepeatedCompositeFieldContainer[TableCell]
    def __init__(self, cells: _Optional[_Iterable[_Union[TableCell, _Mapping]]] = ...) -> None: ...

class Table(_message.Message):
    __slots__ = ("row_count", "column_count", "rows", "column_headers")
    ROW_COUNT_FIELD_NUMBER: _ClassVar[int]
    COLUMN_COUNT_FIELD_NUMBER: _ClassVar[int]
    ROWS_FIELD_NUMBER: _ClassVar[int]
    COLUMN_HEADERS_FIELD_NUMBER: _ClassVar[int]
    row_count: int
    column_count: int
    rows: _containers.RepeatedCompositeFieldContainer[TableRow]
    column_headers: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, row_count: _Optional[int] = ..., column_count: _Optional[int] = ..., rows: _Optional[_Iterable[_Union[TableRow, _Mapping]]] = ..., column_headers: _Optional[_Iterable[str]] = ...) -> None: ...

class ReadTableResponse(_message.Message):
    __slots__ = ("table", "error_message")
    TABLE_FIELD_NUMBER: _ClassVar[int]
    ERROR_MESSAGE_FIELD_NUMBER: _ClassVar[int]
    table: Table
    error_message: str
    def __init__(self, table: _Optional[_Union[Table, _Mapping]] = ..., error_message: _Optional[str] = ...) -> None: ...

class WaitUntilElementExistsRequest(_message.Message):
    __slots__ = ("locator", "timeout_seconds")
    LOCATOR_FIELD_NUMBER: _ClassVar[int]
    TIMEOUT_SECONDS_FIELD_NUMBER: _ClassVar[int]
    locator: str
    timeout_seconds: int
    def __init__(self, locator: _Optional[str] = ..., timeout_seconds: _Optional[int] = ...) -> None: ...

class WaitUntilElementExistsResponse(_message.Message):
    __slots__ = ("exists", "error_message")
    EXISTS_FIELD_NUMBER: _ClassVar[int]
    ERROR_MESSAGE_FIELD_NUMBER: _ClassVar[int]
    exists: bool
    error_message: str
    def __init__(self, exists: bool = ..., error_message: _Optional[str] = ...) -> None: ...

class GetVersionInfoRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class VersionInfo(_message.Message):
    __slots__ = ("vm_version", "bridge_java_class_version", "bridge_java_dll_version", "bridge_win_dll_version")
    VM_VERSION_FIELD_NUMBER: _ClassVar[int]
    BRIDGE_JAVA_CLASS_VERSION_FIELD_NUMBER: _ClassVar[int]
    BRIDGE_JAVA_DLL_VERSION_FIELD_NUMBER: _ClassVar[int]
    BRIDGE_WIN_DLL_VERSION_FIELD_NUMBER: _ClassVar[int]
    vm_version: str
    bridge_java_class_version: str
    bridge_java_dll_version: str
    bridge_win_dll_version: str
    def __init__(self, vm_version: _Optional[str] = ..., bridge_java_class_version: _Optional[str] = ..., bridge_java_dll_version: _Optional[str] = ..., bridge_win_dll_version: _Optional[str] = ...) -> None: ...

class GetVersionInfoResponse(_message.Message):
    __slots__ = ("version_info", "error_message")
    VERSION_INFO_FIELD_NUMBER: _ClassVar[int]
    ERROR_MESSAGE_FIELD_NUMBER: _ClassVar[int]
    version_info: VersionInfo
    error_message: str
    def __init__(self, version_info: _Optional[_Union[VersionInfo, _Mapping]] = ..., error_message: _Optional[str] = ...) -> None: ...

class SubscribeCallbacksRequest(_message.Message):
    __slots__ = ("event_types",)
    EVENT_TYPES_FIELD_NUMBER: _ClassVar[int]
    event_types: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, event_types: _Optional[_Iterable[str]] = ...) -> None: ...

class CallbackEvent(_message.Message):
    __slots__ = ("event_type", "vm_id", "context_handle", "message", "event_time")
    EVENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    VM_ID_FIELD_NUMBER: _ClassVar[int]
    CONTEXT_HANDLE_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    EVENT_TIME_FIELD_NUMBER: _ClassVar[int]
    event_type: str
    vm_id: int
    context_handle: int
    message: str
    event_time: int
    def __init__(self, event_type: _Optional[str] = ..., vm_id: _Optional[int] = ..., context_handle: _Optional[int] = ..., message: _Optional[str] = ..., event_time: _Optional[int] = ...) -> None: ...
