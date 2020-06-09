# Automatically generated by pb2py
# fmt: off
import protobuf as p

if __debug__:
    try:
        from typing import Dict, List  # noqa: F401
        from typing_extensions import Literal  # noqa: F401
    except ImportError:
        pass


class EosActionLinkAuth(p.MessageType):

    def __init__(
        self,
        account: int = None,
        code: int = None,
        type: int = None,
        requirement: int = None,
    ) -> None:
        self.account = account
        self.code = code
        self.type = type
        self.requirement = requirement

    __slots__ = ('account', 'code', 'type', 'requirement',)

    @classmethod
    def get_fields(cls) -> Dict:
        return {
            1: ('account', p.UVarintType, 0),
            2: ('code', p.UVarintType, 0),
            3: ('type', p.UVarintType, 0),
            4: ('requirement', p.UVarintType, 0),
        }
