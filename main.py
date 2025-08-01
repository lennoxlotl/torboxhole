import schedule
import time

from watchdog.nbz_files import check_available_nbz_files
from watchdog.torbox_download_status import check_torbox_download_status

def run():
    create_watchdog_tasks()

    while True:
        schedule.run_pending()
        time.sleep(1)

def create_watchdog_tasks():
    schedule.every(10).seconds.do(check_available_nbz_files)
    schedule.every(10).seconds.do(check_torbox_download_status)

if __name__ == '__main__':
    run()