# Automatically generated by pb2py
# fmt: off
import protobuf as p

from .MoneroTransactionRsigData import MoneroTransactionRsigData

if __debug__:
    try:
        from typing import Dict, List  # noqa: F401
        from typing_extensions import Literal  # noqa: F401
    except ImportError:
        pass


class MoneroTransactionSetOutputAck(p.MessageType):
    MESSAGE_WIRE_TYPE = 512

    def __init__(
        self,
        tx_out: bytes = None,
        vouti_hmac: bytes = None,
        rsig_data: MoneroTransactionRsigData = None,
        out_pk: bytes = None,
        ecdh_info: bytes = None,
    ) -> None:
        self.tx_out = tx_out
        self.vouti_hmac = vouti_hmac
        self.rsig_data = rsig_data
        self.out_pk = out_pk
        self.ecdh_info = ecdh_info

    __slots__ = ('tx_out', 'vouti_hmac', 'rsig_data', 'out_pk', 'ecdh_info',)

    @classmethod
    def get_fields(cls) -> Dict:
        return {
            1: ('tx_out', p.BytesType, 0),
            2: ('vouti_hmac', p.BytesType, 0),
            3: ('rsig_data', MoneroTransactionRsigData, 0),
            4: ('out_pk', p.BytesType, 0),
            5: ('ecdh_info', p.BytesType, 0),
        }
