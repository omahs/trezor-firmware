# Automatically generated by pb2py
# fmt: off
import protobuf as p

if __debug__:
    try:
        from typing import Dict, List  # noqa: F401
        from typing_extensions import Literal  # noqa: F401
        EnumTypeNEMSupplyChangeType = Literal[1, 2]
    except ImportError:
        pass


class NEMMosaicSupplyChange(p.MessageType):

    def __init__(
        self,
        namespace: str = None,
        mosaic: str = None,
        type: EnumTypeNEMSupplyChangeType = None,
        delta: int = None,
    ) -> None:
        self.namespace = namespace
        self.mosaic = mosaic
        self.type = type
        self.delta = delta

    __slots__ = ('namespace', 'mosaic', 'type', 'delta',)

    @classmethod
    def get_fields(cls) -> Dict:
        return {
            1: ('namespace', p.UnicodeType, 0),
            2: ('mosaic', p.UnicodeType, 0),
            3: ('type', p.EnumType("NEMSupplyChangeType", (1, 2)), 0),
            4: ('delta', p.UVarintType, 0),
        }
