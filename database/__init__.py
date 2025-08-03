from sqlalchemy import create_engine
from sqlalchemy.orm import declarative_base, sessionmaker

from config import config

DB_ENGINE = create_engine(f'sqlite:///{config.database_path}')

Base = declarative_base()
Session = sessionmaker(bind=DB_ENGINE)