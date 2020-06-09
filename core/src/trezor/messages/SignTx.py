# Automatically generated by pb2py
# fmt: off
import protobuf as p

if __debug__:
    try:
        from typing import Dict, List  # noqa: F401
        from typing_extensions import Literal  # noqa: F401
    except ImportError:
        pass


class SignTx(p.MessageType):
    MESSAGE_WIRE_TYPE = 15

    def __init__(
        self,
        outputs_count: int = None,
        inputs_count: int = None,
        coin_name: str = None,
        version: int = None,
        lock_time: int = None,
        expiry: int = None,
        version_group_id: int = None,
        timestamp: int = None,
        branch_id: int = None,
    ) -> None:
        self.outputs_count = outputs_count
        self.inputs_count = inputs_count
        self.coin_name = coin_name
        self.version = version
        self.lock_time = lock_time
        self.expiry = expiry
        self.version_group_id = version_group_id
        self.timestamp = timestamp
        self.branch_id = branch_id

    __slots__ = ('outputs_count', 'inputs_count', 'coin_name', 'version', 'lock_time', 'expiry', 'version_group_id', 'timestamp', 'branch_id',)

    @classmethod
    def get_fields(cls) -> Dict:
        return {
            1: ('outputs_count', p.UVarintType, 0),  # required
            2: ('inputs_count', p.UVarintType, 0),  # required
            3: ('coin_name', p.UnicodeType, 0),  # default=Bitcoin
            4: ('version', p.UVarintType, 0),  # default=1
            5: ('lock_time', p.UVarintType, 0),  # default=0
            6: ('expiry', p.UVarintType, 0),
            8: ('version_group_id', p.UVarintType, 0),
            9: ('timestamp', p.UVarintType, 0),
            10: ('branch_id', p.UVarintType, 0),
        }
