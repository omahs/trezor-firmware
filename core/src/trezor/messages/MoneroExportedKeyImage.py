# Automatically generated by pb2py
# fmt: off
import protobuf as p

if __debug__:
    try:
        from typing import Dict, List  # noqa: F401
        from typing_extensions import Literal  # noqa: F401
    except ImportError:
        pass


class MoneroExportedKeyImage(p.MessageType):

    def __init__(
        self,
        iv: bytes = None,
        blob: bytes = None,
    ) -> None:
        self.iv = iv
        self.blob = blob

    __slots__ = ('iv', 'blob',)

    @classmethod
    def get_fields(cls) -> Dict:
        return {
            1: ('iv', p.BytesType, 0),
            3: ('blob', p.BytesType, 0),
        }
