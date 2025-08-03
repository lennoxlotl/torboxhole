import logging
import os
import zipfile
from pathlib import Path

from config import config
from database import Session
from database.nzb import NzbState, NzbDownloadState


def unzip_downloaded_files():
    session = Session()

    try:
        torbox_downloading = session.query(NzbState).filter(NzbState.download_state == NzbDownloadState.DOWNLOADED).all()
        for nzb_state in torbox_downloading:
            logging.info(f"Extracting files in archive of {nzb_state.hash}")
            if not _extract_files(nzb_state):
                continue

            nzb_state.download_state = NzbDownloadState.COMPLETED
            logging.info(f"Completed task of {nzb_state.hash}, media is ready to be used")

        session.commit()
    except:
        session.rollback()
        raise
    finally:
        session.close()
    pass

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

            target_path = os.path.join(target_path, filename)
            with zip_ref.open(zip_info) as source, open(target_path, 'wb') as target:
                target.write(source.read())

    archive_file.unlink()
    return True
