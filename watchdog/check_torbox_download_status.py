import logging

import requests

from config import config
from database import Session
from database.nzb import NzbDownloadState, NzbState
from watchdog import TORBOX_API_VERSION, TORBOX_BASE_URL
from watchdog.decorators import with_db_session


@with_db_session
def check_torbox_download_status(session):
    torbox_downloading = session.query(NzbState).filter(
        NzbState.download_state == NzbDownloadState.TORBOX_DOWNLOADING).all()
    for nzb_state in torbox_downloading:
        if not _fetch_get_download_status(nzb_state):
            continue

        logging.info(
            f"Download of {nzb_state.hash} with download id {nzb_state.download_id} has finished, starting download from CDN")
        nzb_state.download_state = NzbDownloadState.TORBOX_DOWNLOADED


GET_USENET_LIST_URL = "{}/{}/api/usenet/mylist"


def _fetch_get_download_status(nzb_state: NzbState) -> bool:
    """
    Fetches the status of a nzb file download. Does not use Torbox SDK for this as its broken

    :param nzb_state: Internal download state
    :return: Completed or not
    """

    response = requests.request(
        method="GET",
        url=GET_USENET_LIST_URL.format(TORBOX_BASE_URL, TORBOX_API_VERSION),
        params={
            'bypass_cache': 'true',
            'id': str(nzb_state.download_id)
        },
        headers={
            'Authorization': f'Bearer {config.torbox_api_key}'
        }
    )

    json = response.json()

    # Query failed, try again next time
    if not json["success"]:
        return False
    if not json["data"]["download_present"]:
        return False

    return True
