from sqlalchemy import Column, String, Enum, Integer

from database import Base

from enum import Enum as EnumClass


class NzbDownloadState(EnumClass):
    """Defines the state of an ongoing Usenet download"""

    QUEUED = 0
    TORBOX_DOWNLOADING = 1
    TORBOX_DOWNLOADED = 2
    DOWNLOADED = 3
    COMPLETED = 4


class NzbState(Base):
    """Defines the sqlite database model for cached nzb file states"""

    __tablename__ = 'nzb_state'

    hash = Column(String(64), primary_key=True)
    path = Column(String, nullable=False)
    download_state = Column(Enum(NzbDownloadState), nullable=False)
    download_id = Column(Integer, nullable=True)
