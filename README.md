# torboxhole

Automatic downloader for .nbz files using the Torbox API.<br>
Useful while using Usenet Blackholes in Radarr or Sonarr.

## Setup
It is recommended to use the docker image available [here](https://github.com/users/lennoxlotl/packages/container/package/torboxhole).

#### Configuration Variables
These environment variables must be set for the application to function, alternatively a config.yml can be created in the working directory of the application.

- `TBH_NZB_PATH` - Directory to watch for .nzb files
- `TBH_OUTPUT_PATH` - Directory to put final files in
- `TBH_INCOMPLETE_PATH` - Directory to use for downloading and extracting
- `TBH_TORBOX_API_KEY` - Your Torbox API key
- `TBH_DATABASE_PATH` - Where the SQLite database should be created in

## Restrictions
This application currently runs on a single thread meaning parallel downloads are not supported. Right now this is not a big priority for me, as I only want to download one file at a time. Feel free to contribute :)