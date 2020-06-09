# Automatically generated by pb2py
# fmt: off
import protobuf as p

from .EosAuthorizationAccount import EosAuthorizationAccount
from .EosAuthorizationKey import EosAuthorizationKey
from .EosAuthorizationWait import EosAuthorizationWait

if __debug__:
    try:
        from typing import Dict, List  # noqa: F401
        from typing_extensions import Literal  # noqa: F401
    except ImportError:
        pass


class EosAuthorization(p.MessageType):

    def __init__(
        self,
        threshold: int = None,
        keys: List[EosAuthorizationKey] = None,
        accounts: List[EosAuthorizationAccount] = None,
        waits: List[EosAuthorizationWait] = None,
    ) -> None:
        self.threshold = threshold
        self.keys = keys if keys is not None else []
        self.accounts = accounts if accounts is not None else []
        self.waits = waits if waits is not None else []

    __slots__ = ('threshold', 'keys', 'accounts', 'waits',)

    @classmethod
    def get_fields(cls) -> Dict:
        return {
            1: ('threshold', p.UVarintType, 0),
            2: ('keys', EosAuthorizationKey, p.FLAG_REPEATED),
            3: ('accounts', EosAuthorizationAccount, p.FLAG_REPEATED),
            4: ('waits', EosAuthorizationWait, p.FLAG_REPEATED),
        }
