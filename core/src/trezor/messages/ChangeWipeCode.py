# Automatically generated by pb2py
# fmt: off
import protobuf as p

if __debug__:
    try:
        from typing import Dict, List  # noqa: F401
        from typing_extensions import Literal  # noqa: F401
    except ImportError:
        pass


class ChangeWipeCode(p.MessageType):
    MESSAGE_WIRE_TYPE = 82

    def __init__(
        self,
        remove: bool = None,
    ) -> None:
        self.remove = remove

    __slots__ = ('remove',)

    @classmethod
    def get_fields(cls) -> Dict:
        return {
            1: ('remove', p.BoolType, 0),
        }
