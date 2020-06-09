# Automatically generated by pb2py
# fmt: off
import protobuf as p

from .TxRequestDetailsType import TxRequestDetailsType
from .TxRequestSerializedType import TxRequestSerializedType

if __debug__:
    try:
        from typing import Dict, List  # noqa: F401
        from typing_extensions import Literal  # noqa: F401
        EnumTypeRequestType = Literal[0, 1, 2, 3, 4]
    except ImportError:
        pass


class TxRequest(p.MessageType):
    MESSAGE_WIRE_TYPE = 21

    def __init__(
        self,
        request_type: EnumTypeRequestType = None,
        details: TxRequestDetailsType = None,
        serialized: TxRequestSerializedType = None,
    ) -> None:
        self.request_type = request_type
        self.details = details
        self.serialized = serialized

    __slots__ = ('request_type', 'details', 'serialized',)

    @classmethod
    def get_fields(cls) -> Dict:
        return {
            1: ('request_type', p.EnumType("RequestType", (0, 1, 2, 3, 4)), 0),
            2: ('details', TxRequestDetailsType, 0),
            3: ('serialized', TxRequestSerializedType, 0),
        }
