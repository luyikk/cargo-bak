use clap::Parser;
use std::env::var;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{fs, io};
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipWriter};

#[derive(Parser)]
pub enum Opt {
    /// Bak
    /// $CARGO_HOME/bin
    /// $CARGO_HOME/registry/index/
    /// $CARGO_HOME/registry/cache/
    /// $CARGO_HOME/git/db/
    /// to cargo.zip
    Bak {
        #[arg(long, short, value_parser, default_value = "./cargo_bak.zip")]
        save_path: PathBuf,
        #[arg(long, short, value_parser, default_value = "0")]
        compression_level: Option<i32>,
    },
    Restore {
        #[arg(value_parser)]
        path: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    match opt {
        Opt::Bak {
            save_path,
            compression_level,
        } => save_cargo_cache(save_path, compression_level)?,
        Opt::Restore { path } => restore(path)?,
    }

    Ok(())
}

fn save_cargo_cache(save_path: PathBuf, compression_level: Option<i32>) -> anyhow::Result<()> {
    let cargo_home = var("CARGO_HOME")?;
    println!("Start bak CARGO_HOME:{}", cargo_home);
    let bin = PathBuf::from(format!("{}/bin/", cargo_home));
    let registry_index = PathBuf::from(format!("{}/registry/index/", cargo_home));
    let registry_cache = PathBuf::from(format!("{}/registry/cache/", cargo_home));
    let git_db = PathBuf::from(format!("{}/git/db/", cargo_home));
    let mut zip = ZipWriter::new(File::create(save_path)?);

    write_dir(compression_level, &cargo_home, bin, &mut zip)?;
    write_dir(compression_level, &cargo_home, git_db, &mut zip)?;
    write_dir(compression_level, &cargo_home, registry_cache, &mut zip)?;
    write_dir(compression_level, &cargo_home, registry_index, &mut zip)?;

    zip.finish()?;
    println!("finish");
    Ok(())
}

#[inline]
fn write_dir(
    compression_level: Option<i32>,
    cargo_home: &str,
    path: PathBuf,
    zip: &mut ZipWriter<File>,
) -> anyhow::Result<()> {
    let mut buffer = Vec::new();
    for entry in walkdir::WalkDir::new(path) {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let name = path.strip_prefix(cargo_home)?;
            println!("start write file:{}", name.display());
            zip.start_file(
                name.to_string_lossy().to_string(),
                FileOptions::default()
                    .compression_level(compression_level)
                    .compression_method(CompressionMethod::Zstd),
            )?;
            let mut f = File::open(path)?;
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        }
    }
    Ok(())
}

fn restore(file: PathBuf) -> anyhow::Result<()> {
    if !file.exists() {
        println!("not found:{}", file.display());
        return Ok(());
    }

    let cargo_home = var("CARGO_HOME")?;
    let file = File::open(file)?;
    let mut archive = zip::ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => PathBuf::from(format!("{}/{}", cargo_home, path.display())),
            None => continue,
        };

        {
            let comment = file.comment();
            if !comment.is_empty() {
                println!("File {i} comment: {comment}");
            }
        }

        if (*file.name()).ends_with('/') {
            println!("File {} extracted to \"{}\"", i, outpath.display());
            fs::create_dir_all(&outpath)?;
        } else {
            println!(
                "File {} extracted to \"{}\" ({} bytes)",
                i,
                outpath.display(),
                file.size()
            );
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
            }
        }
    }

    Ok(())
}
