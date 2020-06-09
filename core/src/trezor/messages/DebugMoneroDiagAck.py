# Automatically generated by pb2py
# fmt: off
import protobuf as p

if __debug__:
    try:
        from typing import Dict, List  # noqa: F401
        from typing_extensions import Literal  # noqa: F401
    except ImportError:
        pass


class DebugMoneroDiagAck(p.MessageType):
    MESSAGE_WIRE_TYPE = 547

    def __init__(
        self,
        ins: int = None,
        p1: int = None,
        p2: int = None,
        pd: List[int] = None,
        data1: bytes = None,
        data2: bytes = None,
    ) -> None:
        self.ins = ins
        self.p1 = p1
        self.p2 = p2
        self.pd = pd if pd is not None else []
        self.data1 = data1
        self.data2 = data2

    __slots__ = ('ins', 'p1', 'p2', 'pd', 'data1', 'data2',)

    @classmethod
    def get_fields(cls) -> Dict:
        return {
            1: ('ins', p.UVarintType, 0),
            2: ('p1', p.UVarintType, 0),
            3: ('p2', p.UVarintType, 0),
            4: ('pd', p.UVarintType, p.FLAG_REPEATED),
            5: ('data1', p.BytesType, 0),
            6: ('data2', p.BytesType, 0),
        }
