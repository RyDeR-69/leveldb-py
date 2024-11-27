import typing

__version__: str
__doc__: str
__all__: list[str]

class Database:
    def __init__(self, path: str, open_options: OpenOptions) -> None: ...
    def iter(self, read_options: ReadOptions) -> Iterator: ...

class Iterator(typing.Iterator[typing.Tuple[bytes, bytes]]):
    def __iter__(self): ...
    def __next__(self) -> typing.Optional[typing.Tuple[bytes, bytes]]: ...

class OpenOptions:
    def __init__(
        self,
        create_if_missing: bool = False,
        error_if_exists: bool = False,
        paranoid_checks: bool = False,
        write_buffer_size: typing.Optional[int] = None,
        max_open_files: typing.Optional[int] = None,
        block_size: typing.Optional[int] = None,
        block_restart_interval: typing.Optional[int] = None,
        compression: bool = False,
        cache: typing.Optional[int] = None,
    ) -> None: ...

class ReadOptions:
    def __init__(self, verify_checksums: bool = False, fill_cache: bool = True) -> None: ...

class WriteOptions:
    def __init__(self, sync: bool = False) -> None: ...

PyOpenOptionsTypes = typing.Union[OpenOptions]
PyReadOptionsTypes = typing.Union[ReadOptions]
PyWriteOptionsTypes = typing.Union[WriteOptions]
