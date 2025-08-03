import logging
import os.path
from pathlib import Path

import requests

from config import config
from database import Session
from database.nzb import NzbState, NzbDownloadState
from watchdog import TORBOX_API_VERSION, TORBOX_BASE_URL


def start_queued_torbox_downloads():
    session = Session()

    try:
        torbox_downloading  = session.query(NzbState).filter(NzbState.download_state == NzbDownloadState.TORBOX_DOWNLOADED).all()
        for nzb_state in torbox_downloading:
            link = _get_torbox_cdn_link(nzb_state)

            logging.info(f"Starting/Resuming download of {nzb_state.hash} with download id {nzb_state.download_id}")
            _start_download(nzb_state, link)
            logging.info(f"Finished downloading {nzb_state.hash} with download id {nzb_state.download_id}")

            nzb_state.download_state = NzbDownloadState.DOWNLOADED

        session.commit()
    except:
        session.rollback()
        raise
    finally:
        session.close()

GET_USENET_DOWNLOAD_LINK = "{}/{}/api/usenet/requestdl"

def _get_torbox_cdn_link(nzb_state: NzbState) -> str:
    """
        Fetches the status of a nzb file download. Does not use Torbox SDK for this as its broken

        :param nzb_state: Internal download state
        :return: Completed or not
        """

    response = requests.request(
        method="GET",
        url=GET_USENET_DOWNLOAD_LINK.format(TORBOX_BASE_URL, TORBOX_API_VERSION),
        params={
            'token': config.torbox_api_key,
            'zip_link': 'true',
            'usenet_id': str(nzb_state.download_id)
        },
        headers={
            'Authorization': f'Bearer {config.torbox_api_key}'
        }
    )

    json = response.json()

    # Query failed, try again next time
    if not json["success"]:
        return ""

    return json["data"]

CHUNK_SIZE = 1024 * 1024

def _start_download(nzb_state: NzbState, link: str):
    part_file = Path(f"{config.incomplete_path}/{nzb_state.hash}.part")

    # Delete old downloads
    if os.path.exists(part_file):
        part_file.unlink()

    with requests.get(link, stream=True) as response:
        response.raise_for_status()
        # Write file in chunks
        with open(part_file, "wb") as f:
            for chunk in response.iter_content(chunk_size=CHUNK_SIZE):
                f.write(chunk)

    os.rename(part_file, f"{config.incomplete_path}/{nzb_state.hash}.zip")