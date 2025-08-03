import logging
import time

import schedule

import database
from watchdog.check_torbox_download_status import check_torbox_download_status
from watchdog.nbz_files import check_available_nbz_files
from watchdog.queue_torbox_download import queue_torbox_downloads
from watchdog.start_queued_torbox_downloads import start_queued_torbox_downloads
from watchdog.unzip_downloaded_files import unzip_downloaded_files


def run():
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(levelname)s - %(message)s',
    )
    database.Base.metadata.create_all(database.DB_ENGINE)
    create_watchdog_tasks()

    while True:
        schedule.run_pending()
        time.sleep(1)


def create_watchdog_tasks():
    schedule.every(1).seconds.do(check_available_nbz_files)

    # API call intensive tasks, shall only be called rarely
    schedule.every(10).seconds.do(queue_torbox_downloads)
    schedule.every(10).seconds.do(check_torbox_download_status)
    schedule.every(10).seconds.do(start_queued_torbox_downloads)

    schedule.every(1).seconds.do(unzip_downloaded_files)


if __name__ == '__main__':
    run()
