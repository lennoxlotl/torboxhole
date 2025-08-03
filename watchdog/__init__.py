from torbox_api import TorboxApi

from config import config

TORBOX_BASE_URL = "https://api.torbox.app"
TORBOX_API_VERSION = "v1"

TORBOX_SDK = TorboxApi(
    access_token=config.torbox_api_key,
    base_url = TORBOX_BASE_URL,
    timeout=10000
)