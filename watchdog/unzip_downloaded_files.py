import logging
import os
import zipfile
from pathlib import Path

from config import config
from database.nzb import NzbState, NzbDownloadState
from watchdog.decorators import with_db_session


@with_db_session
def unzip_downloaded_files(session):
    torbox_downloading = session.query(NzbState).filter(NzbState.download_state == NzbDownloadState.DOWNLOADED).all()
    for nzb_state in torbox_downloading:
        logging.info(f"Extracting files in archive of {nzb_state.hash}")
        if not _extract_files(nzb_state):
            continue

        nzb_state.download_state = NzbDownloadState.COMPLETED
        logging.info(f"Completed task of {nzb_state.hash}, media is ready to be used")


CHUNK_SIZE = 64 * 1024


def _extract_files(nzb_state: NzbState) -> bool:
    """
    Extracts all files of the downloaded Usenet ZIP archive into the output folder flattened.

    :param nzb_state: Internal download state
    :return: Success state
    """

    archive_file = Path(f"{config.incomplete_path}/{nzb_state.hash}.zip")
    target_path = Path(f"{config.output_path}")

    with zipfile.ZipFile(archive_file) as zip_ref:
        for zip_info in zip_ref.infolist():
            # Ignore directories, it will still go over the files in this directory
            if zip_info.is_dir():
                continue

            original_path = zip_info.filename
            filename = os.path.basename(original_path)
            # Invalid filename, skip this file
            if not filename:
                continue

            file_path = os.path.join(target_path, filename)
            if os.path.exists(file_path):
                continue

            with zip_ref.open(zip_info) as source, open(file_path, 'wb') as target:
                while True:
                    chunk = source.read(CHUNK_SIZE)
                    if not chunk:
                        break
                    target.write(chunk)

    archive_file.unlink()
    return True
