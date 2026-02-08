use eyre::eyre;
use log::{error, info, warn};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use sanitize_filename::sanitize;
use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use tbh_database::download::{
    find_downloads_with_state, set_download_lock, set_download_state, Download, DownloadState,
};
use zip::ZipArchive;

const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg", "ts", "3gp", "ogv",
];

pub async fn process_extract(
    pool: Pool<SqliteConnectionManager>,
    download_dir: &PathBuf,
    output_dir: &PathBuf,
) {
    match extract_downloaded_files(pool, download_dir, output_dir).await {
        Ok(_) => {}
        Err(err) => {
            log::error!("FATAL: Unable to extract downloads: {}", err);
        }
    }
}

async fn extract_downloaded_files(
    pool: Pool<SqliteConnectionManager>,
    download_dir: &PathBuf,
    output_dir: &PathBuf,
) -> eyre::Result<()> {
    let conn = pool.get()?;

    for download in find_downloads_with_state(&conn, DownloadState::Completed)? {
        if let Err(e) = handle_extract(download.clone(), &pool, &download_dir, &output_dir).await {
            error!("Error extracting download {}: {}", download.name, e);
            let conn = match pool.get() {
                Ok(c) => c,
                Err(_) => continue,
            };
            let _ = set_download_lock(&conn, download.id, false);
            set_download_state(&conn, download.id, DownloadState::Failed)?;
        }
    }

    Ok(())
}

async fn handle_extract(
    download: Download,
    pool: &Pool<SqliteConnectionManager>,
    download_dir: &Path,
    output_dir: &Path,
) -> eyre::Result<()> {
    let conn = pool.get()?;

    if download.lock {
        return Ok(());
    }
    set_download_lock(&conn, download.id, true)?;
    set_download_state(&conn, download.id, DownloadState::Extracting)?;
    info!("Starting extraction for: {}", download.name);

    let file_name = sanitize(&download.name) + ".zip";
    let file_path = download_dir.join(&file_name);
    if !file_path.exists() {
        set_download_lock(&conn, download.id, false)?;
        return Err(eyre!("File not found: {}", file_path.display()));
    }

    let target_dir = output_dir.to_path_buf();
    let temp_extract_dir = output_dir.join(format!("_temp_{}", sanitize(&download.name)));
    if temp_extract_dir.exists() {
        fs::remove_dir_all(&temp_extract_dir)?;
    }
    fs::create_dir_all(&temp_extract_dir)?;

    if let Err(e) = extract_and_process_internal(&file_path, &temp_extract_dir) {
        set_download_lock(&conn, download.id, false)?;
        let _ = fs::remove_dir_all(&temp_extract_dir);
        return Err(eyre!("Failed to extract file: {}", e));
    }
    if let Err(e) = flatten_files(&temp_extract_dir, &target_dir) {
        set_download_lock(&conn, download.id, false)?;
        let _ = fs::remove_dir_all(&temp_extract_dir);
        return Err(eyre!("Failed to flatten files: {}", e));
    }

    let _ = fs::remove_dir_all(&temp_extract_dir);
    let _ = fs::remove_file(&file_path);
    set_download_state(&conn, download.id, DownloadState::Extracted)?;
    set_download_lock(&conn, download.id, false)?;
    info!("Finished extraction: {}", download.name);

    Ok(())
}

fn flatten_files(source: &Path, destination: &Path) -> eyre::Result<()> {
    if !source.exists() {
        return Ok(());
    }

    let entries = fs::read_dir(source)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            flatten_files(&path, destination)?;
        } else if path.is_file() {
            if is_video_file(&path) {
                let file_name = path.file_name().ok_or_else(|| eyre!("No filename"))?;
                let dest_path = destination.join(file_name);

                if dest_path.exists() {
                    fs::remove_file(&dest_path)?;
                }
                fs::rename(&path, &dest_path)?;
            }
        }
    }
    Ok(())
}

fn extract_and_process_internal(zip_path: &Path, output_dir: &Path) -> eyre::Result<()> {
    extract_zip(zip_path, output_dir)?;
    process_directory(output_dir)?;
    Ok(())
}

fn is_video_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        if let Some(ext_str) = ext.to_str() {
            let ext_lower = ext_str.to_lowercase();
            return VIDEO_EXTENSIONS.contains(&ext_lower.as_str());
        }
    }
    false
}

fn process_directory(dir: &Path) -> eyre::Result<()> {
    // Try to handle nested zip files, 5 iterations should be enough in theory (I hope)
    for _ in 0..5 {
        let entries = fs::read_dir(dir)?;
        let mut zips_to_process = Vec::new();

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "zip" {
                        zips_to_process.push(path);
                    }
                }
            } else if path.is_dir() {
                process_directory(&path)?;
            }
        }

        for zip_path in zips_to_process {
            let file_stem = zip_path.file_stem().unwrap().to_string_lossy();
            let parent = zip_path.parent().unwrap();
            let part_file = parent.join(format!("{}.z01", file_stem));

            if part_file.exists() {
                combine_and_extract(&zip_path, parent)?;
                fs::remove_file(&zip_path)?;
                continue;
            } else {
                match extract_zip(&zip_path, parent) {
                    Ok(_) => {
                        fs::remove_file(&zip_path)?;
                        continue;
                    }
                    Err(e) => {
                        warn!("Failed to extract nested zip {}: {}", zip_path.display(), e);
                    }
                }
            }
        }

        // No changes were made, we can stop extracting
        break;
    }
    Ok(())
}

fn combine_and_extract(main_zip_path: &Path, output_dir: &Path) -> eyre::Result<()> {
    let file_stem = main_zip_path.file_stem().unwrap().to_string_lossy();
    let parent = main_zip_path.parent().unwrap();
    let merged_path = parent.join(format!("{}_merged.zip", file_stem));
    let mut merged_file = File::create(&merged_path)?;

    // Try until no more part-files can be found
    let mut part_file_count = 1;
    loop {
        let part_path = parent.join(format!("{}.z{:02}", file_stem, part_file_count));
        if !part_path.exists() {
            break;
        }
        let mut part_file = File::open(&part_path)?;
        io::copy(&mut part_file, &mut merged_file)?;
        let _ = fs::remove_file(&part_path);
        part_file_count += 1;
    }

    let mut main_file = File::open(main_zip_path)?;
    io::copy(&mut main_file, &mut merged_file)?;
    extract_zip(&merged_path, output_dir)?;
    fs::remove_file(merged_path)?;

    Ok(())
}

fn extract_zip(zip_path: &Path, output_dir: &Path) -> eyre::Result<()> {
    let file = File::open(zip_path)?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = match file.enclosed_name() {
            Some(path) => output_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(p) = out_path.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            io::copy(&mut file, &mut File::create(&out_path)?)?;
        }
    }
    Ok(())
}
