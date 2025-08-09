import os.path

import yaml

CONFIG_PATH: str = 'config.yml'


class TorboxHoleConfig:
    """Config file as model class, parses input dictionaries"""

    nzb_path: str
    output_path: str
    incomplete_path: str
    torbox_api_key: str
    database_path: str
    concurrent_download_limit: int

    # noinspection PyShadowingNames
    def __init__(self, config: dict):
        self.nzb_path = config['nzb_path']
        self.output_path = config['output_path']
        self.incomplete_path = config['incomplete_path']
        self.torbox_api_key = config['torbox_api_key']
        self.database_path = config['database_path']
        self.concurrent_download_limit = int(config['concurrent_download_limit'])


def load_config() -> TorboxHoleConfig:
    if not os.path.exists(CONFIG_PATH):
        # Use environment variables if config is not available
        return TorboxHoleConfig({
            'nzb_path': os.environ.get('TBH_NZB_PATH'),
            'output_path': os.environ.get('TBH_OUTPUT_PATH'),
            'incomplete_path': os.environ.get('TBH_INCOMPLETE_PATH'),
            'torbox_api_key': os.environ.get('TBH_TORBOX_API_KEY'),
            'database_path': os.environ.get('TBH_DATABASE_PATH'),
            'concurrent_download_limit': os.environ.get('TBH_CONCURRENT_DOWNLOAD_LIMIT'),
        })

    with open(CONFIG_PATH, 'r') as f:
        return TorboxHoleConfig(yaml.safe_load(f))


config: TorboxHoleConfig = load_config()
