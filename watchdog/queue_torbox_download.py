import logging
from pathlib import Path

from torbox_api.models import CreateUsenetDownloadRequest

from config import config
from database import Session
from database.nzb import NzbState, NzbDownloadState
from watchdog import TORBOX_SDK, TORBOX_API_VERSION
from watchdog.decorators import with_db_session
from sqlalchemy import func


class StartTorboxDownloadResult:
    """Represents a simplified result of a torbox download queue start"""

    success: bool
    ratelimit_reached: bool
    download_id: int

    def __init__(self, success: bool, ratelimit_reached: bool, download_id: int):
        self.success = success
        self.ratelimit_reached = ratelimit_reached
        self.download_id = download_id

    @classmethod
    def failed(cls):
        return cls(False, False, 0)

    @classmethod
    def ratelimit(cls):
        return cls(False, True, 0)

    @classmethod
    def succeeded(cls, download_id: int):
        return cls(True, False, download_id)


@with_db_session
def queue_torbox_downloads(session):
    # Don't queue download if max limit is reached
    start_download_count = session.query(NzbState).filter(
        NzbState.download_state == NzbDownloadState.TORBOX_DOWNLOADING).count()
    logging.info(f"Currently active downloads: {start_download_count}")
    if start_download_count >= config.concurrent_download_limit - 1:
        return

    queued_downloads = session.query(NzbState).filter(NzbState.download_state == NzbDownloadState.QUEUED).all()
    for nzb_state in queued_downloads:
        # noinspection PyBroadException
        try:
            result = _try_start_torbox_download(nzb_state)

            # Abort any future downloads if concurrent download limit is reached
            start_download_count += 1
            if start_download_count >= config.concurrent_download_limit - 1:
                logging.info("Reached rate-limit continuing later...")
                break

            # Download started, continue
            if result.success:
                logging.info(
                    f"Queued download for {nzb_state.hash}, has download id {result.download_id}, deleted .nzb file")
                nzb_state.download_id = result.download_id
                nzb_state.download_state = NzbDownloadState.TORBOX_DOWNLOADING
                # .nbz source file is not required anymore, Torbox has received it successfully
                Path(nzb_state.path.decode("UTF-8")).unlink()
                continue

            # Error is unrelated to rate-limit, delete the file and abort
            if not result.ratelimit_reached:
                Path(nzb_state.path.decode("UTF-8")).unlink()
                session.delete(nzb_state)
        # Allow universal except here, if anything goes wrong the download should be aborted
        except:
            logging.error(
                "An error occurred while trying to start torbox download, please queue the file again if not automatically detected")
            session.delete(nzb_state)


def _try_start_torbox_download(state: NzbState) -> StartTorboxDownloadResult:
    """
    Tries starting a torbox download by uploading the nzb file.

    :param state: Internal download state
    :return: Result of download
    """

    with open(state.path, 'rb') as f:
        nzb_data = f.read()

    response = (
        TORBOX_SDK.usenet.create_usenet_download(
            TORBOX_API_VERSION,
            CreateUsenetDownloadRequest(nzb_data)
        )
    )

    if not response.success:
        # If download failed because of link errors, force deletion, otherwise we assume it's a rate-limit
        match response.error:
            case "LINK_OFFLINE", "DOWNLOAD_TOO_LARGE", "TOO_MUCH_DATA":
                return StartTorboxDownloadResult.failed()
            case _:
                return StartTorboxDownloadResult.ratelimit()

    return StartTorboxDownloadResult.succeeded(int(response.data.usenetdownload_id))


# noinspection PyTypeHints
def _downloading_count(session: Session) -> int:
    return session.query(func.count(NzbState.id)).filter(
        NzbState.download_state == NzbDownloadState.TORBOX_DOWNLOADING).scalar()
