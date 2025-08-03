import hashlib
import logging
from pathlib import Path

from config import config
from database import Session
from database.nzb import NzbState, NzbDownloadState


def check_available_nbz_files():
    """
    Checks for yet unknown NZB files to automatically queue them for download.
    """

    nzb_path = Path(config.nzb_path)
    for file in nzb_path.iterdir():
        # Recursive indexing is not supported
        if not file.is_file():
            continue
        # Only .nzb files are supported
        if file.suffix != '.nzb':
            continue

        check_nzb_file(file)


def check_nzb_file(file):
    # Hashing the file name is enough here, duplicates are usually prevented by *arr stack applications
    file_hash = hashlib.sha256(str(file.name).encode()).hexdigest()
    session = Session()

    try:
        state = session.query(NzbState).filter(NzbState.hash == file_hash).first()
        # Queue nzb file if not present
        if state is None:
            logging.info(f"Queueing file {file.name} with hash {file_hash} for download")
            state = NzbState(hash=file_hash, path=file.resolve().as_posix().encode(), download_state=NzbDownloadState.QUEUED)
            session.add(state)
            session.commit()
    except:
        session.rollback()
        raise
    finally:
        session.close()
