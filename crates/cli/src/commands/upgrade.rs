use crate::error::ProtoCliError;
use crate::helpers::download_to_temp_with_progress_bar;
use miette::IntoDiagnostic;
use proto_core::{is_offline, ProtoEnvironment};
use semver::Version;
use starbase::system;
use starbase_archive::Archiver;
use starbase_styles::color;
use starbase_utils::fs;
use std::env::consts;
use std::path::PathBuf;
use tracing::{debug, info, warn};

async fn fetch_version() -> miette::Result<String> {
    let version = reqwest::get("https://raw.githubusercontent.com/moonrepo/proto/master/version")
        .await
        .into_diagnostic()?
        .text()
        .await
        .into_diagnostic()?
        .trim()
        .to_string();

    debug!("Found latest version {}", color::hash(&version));

    Ok(version)
}

#[system]
pub async fn upgrade() {
    if is_offline() {
        return Err(ProtoCliError::UpgradeRequiresInternet.into());
    }

    let proto = ProtoEnvironment::new()?;
    let current_version = env!("CARGO_PKG_VERSION");
    let new_version = fetch_version().await?;

    debug!(
        "Comparing latest version {} to local version {}",
        color::hash(&new_version),
        color::hash(current_version),
    );

    if Version::parse(&new_version).unwrap() <= Version::parse(current_version).unwrap() {
        info!("You're already on the latest version of proto!");

        return Ok(());
    }

    // Determine the download file based on target
    let target = match (consts::OS, consts::ARCH) {
        ("linux", arch) => format!("{arch}-unknown-linux-gnu"),
        ("macos", arch) => format!("{arch}-apple-darwin"),
        ("windows", "x86_64") => "x86_64-pc-windows-msvc".to_owned(),
        (os, arch) => {
            return Err(ProtoCliError::UpgradeInvalidPlatform {
                arch: arch.to_owned(),
                os: os.to_owned(),
            }
            .into());
        }
    };
    let target_ext = if cfg!(windows) { "zip" } else { "tar.xz" };
    let target_file = format!("proto_cli-{target}");

    debug!("Download target: {}", &target_file);

    // Download the file and show a progress bar
    let download_file = format!("{target_file}.{target_ext}");
    let download_url = format!(
        "https://github.com/moonrepo/proto/releases/download/v{new_version}/{download_file}"
    );
    let temp_file = download_to_temp_with_progress_bar(&download_url, &download_file).await?;
    let temp_dir = proto.temp_dir.join(&target_file);

    // Unpack the downloaded file
    Archiver::new(&temp_dir, &temp_file).unpack_from_ext()?;

    // Move the old binary
    let bin_name = if cfg!(windows) { "proto.exe" } else { "proto" };
    let bin_path = proto.bin_dir.join(bin_name);

    if bin_path.exists() {
        fs::rename(
            &bin_path,
            proto
                .tools_dir
                .join("proto")
                .join(current_version)
                .join(bin_name),
        )?;
    }

    // Move the new binary to the bins directory
    let lookup_paths = vec![
        PathBuf::from(target_file).join(bin_name),
        PathBuf::from(bin_name),
    ];

    for lookup_path in lookup_paths {
        let possible_bin_path = temp_dir.join(lookup_path);

        if possible_bin_path.exists() {
            fs::copy_file(possible_bin_path, &bin_path)?;
            fs::update_perms(&bin_path, None)?;

            fs::remove(temp_dir)?;
            fs::remove(temp_file)?;

            info!("Upgraded proto to v{}!", new_version);
            warn!("Changes to PATH were made in v0.20. Please refer to the changelog and migration guide!");

            return Ok(());
        }
    }

    Err(ProtoCliError::UpgradeFailed {
        bin: bin_name.to_owned(),
    })?;
}
