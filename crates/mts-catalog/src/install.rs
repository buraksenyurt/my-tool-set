//! Installs release tools under `<tools>/<id>/<version>/`.
//!
//! An install is all or nothing:
//!
//! 1. download into `<tools>/<id>/.partial-<version>/` while hashing,
//! 2. check size and SHA256 against `release.json`,
//! 3. unpack into the same scratch folder and check that `entry` exists,
//! 4. write the `.mts-install.json` marker, then rename the folder into place.
//!
//! Only folders with a marker count as installed, so a crash or a failed check
//! at any step leaves nothing that looks usable.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use mts_core::release::{PlatformRelease, PortableAsset};
use mts_core::{ArchiveFormat, Platform, ReleaseManifest, Version};
use serde::{Deserialize, Serialize};

use crate::CatalogError;
use crate::cache::now;
use crate::error::IoContext;
use crate::releases::ReleaseClient;

const MARKER: &str = ".mts-install.json";
const PARTIAL_PREFIX: &str = ".partial-";

/// What the marker file records about an install.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallRecord {
    pub tool: String,
    pub version: String,
    pub tag: String,
    pub repo: String,
    pub platform: String,
    pub asset: String,
    pub sha256: String,
    pub entry: String,
    /// Seconds since the Unix epoch.
    pub installed_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Installed {
    pub record: InstallRecord,
    pub folder: PathBuf,
    /// Absolute path of the file to run.
    pub entry: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Downloading { received: u64, total: u64 },
    Verifying,
    Unpacking,
}

#[derive(Debug, Clone)]
pub struct Installer {
    tools_dir: PathBuf,
}

impl Installer {
    pub fn new(tools_dir: impl Into<PathBuf>) -> Self {
        Self {
            tools_dir: tools_dir.into(),
        }
    }

    fn tool_dir(&self, tool: &str) -> PathBuf {
        self.tools_dir.join(tool)
    }

    /// Installed versions of `tool`, newest first.
    pub fn installed(&self, tool: &str) -> Vec<Installed> {
        let Ok(read) = fs::read_dir(self.tool_dir(tool)) else {
            return Vec::new();
        };
        let mut found: Vec<Installed> = read
            .filter_map(|e| e.ok())
            .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
            .filter_map(|e| {
                let folder = e.path();
                let record: InstallRecord =
                    serde_json::from_slice(&fs::read(folder.join(MARKER)).ok()?).ok()?;
                let entry = folder.join(&record.entry);
                Some(Installed {
                    record,
                    folder,
                    entry,
                })
            })
            .collect();
        found.sort_by(|a, b| {
            Version::parse(&b.record.version).cmp(&Version::parse(&a.record.version))
        });
        found
    }

    /// Downloads and installs `release` for `platform`. Installing a version
    /// that is already there returns the existing install untouched.
    pub async fn install(
        &self,
        client: &ReleaseClient,
        release: &ReleaseManifest,
        platform: Platform,
        on_step: &(dyn Fn(Step) + Send + Sync),
    ) -> Result<Installed, CatalogError> {
        let build = pick(release, platform)?;
        if let Some(existing) = self
            .installed(&release.tool)
            .into_iter()
            .find(|i| i.record.version == release.version)
        {
            return Ok(existing);
        }

        let scratch = self.scratch_dir(release)?;
        let download = scratch.join(&build.portable.asset);
        let url = client.asset_url(&release.repo, &release.tag, &build.portable.asset);
        let result = async {
            let (sha, size) = client
                .download(&url, &download, build.portable.size, &|received, total| {
                    on_step(Step::Downloading { received, total })
                })
                .await?;
            on_step(Step::Verifying);
            verify(&build.portable, &sha, size)?;
            on_step(Step::Unpacking);
            let installer = self.clone();
            let release = release.clone();
            let build = build.clone();
            tokio::task::spawn_blocking(move || {
                installer.finish(&release, platform, &build, &scratch, &download)
            })
            .await
            .expect("unpack task panicked")
        }
        .await;
        if result.is_err() {
            let _ = fs::remove_dir_all(self.scratch_path(release));
        }
        result
    }

    /// Steps 3–4 for a file that is already downloaded and verified. Public so
    /// it can be tested without a network.
    pub fn install_downloaded(
        &self,
        release: &ReleaseManifest,
        platform: Platform,
        download: &Path,
    ) -> Result<Installed, CatalogError> {
        let build = pick(release, platform)?.clone();
        let (sha, size) = hash_file(download)?;
        verify(&build.portable, &sha, size)?;
        let scratch = self.scratch_dir(release)?;
        let copy = scratch.join(&build.portable.asset);
        fs::copy(download, &copy).at(&copy)?;
        let result = self.finish(release, platform, &build, &scratch, &copy);
        if result.is_err() {
            let _ = fs::remove_dir_all(&scratch);
        }
        result
    }

    pub fn uninstall(&self, tool: &str, version: &str) -> Result<(), CatalogError> {
        let install = self
            .installed(tool)
            .into_iter()
            .find(|i| i.record.version == version)
            .ok_or_else(|| CatalogError::NotInstalled {
                tool: tool.into(),
                version: version.into(),
            })?;
        // Remove the marker first: if deleting the rest fails half way (a file
        // still in use on Windows), the folder no longer counts as installed.
        let marker = install.folder.join(MARKER);
        fs::remove_file(&marker).at(&marker)?;
        fs::remove_dir_all(&install.folder).at(&install.folder)
    }

    fn scratch_path(&self, release: &ReleaseManifest) -> PathBuf {
        self.tool_dir(&release.tool)
            .join(format!("{PARTIAL_PREFIX}{}", release.version))
    }

    /// A fresh, empty scratch folder (leftovers of an earlier attempt removed).
    fn scratch_dir(&self, release: &ReleaseManifest) -> Result<PathBuf, CatalogError> {
        let scratch = self.scratch_path(release);
        if scratch.exists() {
            fs::remove_dir_all(&scratch).at(&scratch)?;
        }
        fs::create_dir_all(&scratch).at(&scratch)?;
        Ok(scratch)
    }

    fn finish(
        &self,
        release: &ReleaseManifest,
        platform: Platform,
        build: &PlatformRelease,
        scratch: &Path,
        download: &Path,
    ) -> Result<Installed, CatalogError> {
        let p = &build.portable;
        let content = scratch.join("content");
        fs::create_dir_all(&content).at(&content)?;
        unpack(p, download, &content)?;
        fs::remove_file(download).at(download)?;

        let entry = content.join(&p.entry);
        if !entry.is_file() {
            return Err(CatalogError::EntryMissing(p.entry.clone()));
        }
        make_executable(&entry)?;

        let record = InstallRecord {
            tool: release.tool.clone(),
            version: release.version.clone(),
            tag: release.tag.clone(),
            repo: release.repo.clone(),
            platform: platform.key(),
            asset: p.asset.clone(),
            sha256: p.sha256.to_ascii_lowercase(),
            entry: p.entry.clone(),
            installed_at: now(),
        };
        let marker = content.join(MARKER);
        fs::write(
            &marker,
            serde_json::to_vec_pretty(&record).expect("serializable"),
        )
        .at(&marker)?;

        let folder = self.tool_dir(&release.tool).join(&release.version);
        if folder.exists() {
            // A folder without a marker: leftovers, not an install.
            fs::remove_dir_all(&folder).at(&folder)?;
        }
        rename_with_retry(&content, &folder).at(&folder)?;
        let _ = fs::remove_dir_all(scratch);

        Ok(Installed {
            entry: folder.join(&record.entry),
            record,
            folder,
        })
    }
}

/// On Windows, antivirus scanners briefly lock freshly written executables, and
/// renaming their folder fails with "access denied". Retry a few times.
fn rename_with_retry(from: &Path, to: &Path) -> io::Result<()> {
    let mut attempt = 0;
    loop {
        match fs::rename(from, to) {
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied && attempt < 10 => {
                attempt += 1;
                std::thread::sleep(std::time::Duration::from_millis(150 * attempt));
            }
            other => return other,
        }
    }
}

fn pick(release: &ReleaseManifest, platform: Platform) -> Result<&PlatformRelease, CatalogError> {
    let problems = release.validate(&release.tool, &release.repo);
    if !problems.is_empty() {
        return Err(CatalogError::InvalidRelease(problems.join("; ")));
    }
    release
        .for_platform(platform)
        .ok_or_else(|| CatalogError::NoBuildForPlatform {
            tool: release.tool.clone(),
            platform: platform.key(),
        })
}

fn verify(p: &PortableAsset, sha: &str, size: u64) -> Result<(), CatalogError> {
    if size != p.size {
        return Err(CatalogError::SizeMismatch {
            asset: p.asset.clone(),
            expected: p.size,
            actual: size,
        });
    }
    if !sha.eq_ignore_ascii_case(&p.sha256) {
        return Err(CatalogError::ShaMismatch {
            asset: p.asset.clone(),
            expected: p.sha256.clone(),
            actual: sha.into(),
        });
    }
    Ok(())
}

fn hash_file(path: &Path) -> Result<(String, u64), CatalogError> {
    use io::Read;
    use sha2::{Digest, Sha256};
    let mut file = fs::File::open(path).at(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    let mut size = 0u64;
    loop {
        let n = file.read(&mut buf).at(path)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        size += n as u64;
    }
    Ok((hex::encode(hasher.finalize()), size))
}

fn unpack(p: &PortableAsset, archive: &Path, dest: &Path) -> Result<(), CatalogError> {
    let archive_error = |message: String| CatalogError::Archive {
        asset: p.asset.clone(),
        message,
    };
    match p.format {
        ArchiveFormat::File => {
            let target = dest.join(&p.asset);
            fs::copy(archive, &target).at(&target)?;
        }
        ArchiveFormat::Zip => {
            let file = fs::File::open(archive).at(archive)?;
            let mut zip = zip::ZipArchive::new(file).map_err(|e| archive_error(e.to_string()))?;
            for i in 0..zip.len() {
                let mut item = zip.by_index(i).map_err(|e| archive_error(e.to_string()))?;
                // enclosed_name() is None for absolute paths and `..`: zip-slip.
                let Some(relative) = item.enclosed_name() else {
                    let name = item.name().map(|n| n.into_owned()).unwrap_or_default();
                    return Err(archive_error(format!("güvensiz yol: {name}")));
                };
                let target = dest.join(relative);
                if item.is_dir() {
                    fs::create_dir_all(&target).at(&target)?;
                    continue;
                }
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent).at(parent)?;
                }
                let mut out = fs::File::create(&target).at(&target)?;
                io::copy(&mut item, &mut out).at(&target)?;
                #[cfg(unix)]
                if let Some(mode) = item.unix_mode() {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(&target, fs::Permissions::from_mode(mode & 0o777))
                        .at(&target)?;
                }
            }
        }
        ArchiveFormat::TarGz => {
            let file = fs::File::open(archive).at(archive)?;
            let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(file));
            tar.set_preserve_permissions(true);
            for item in tar.entries().map_err(|e| archive_error(e.to_string()))? {
                let mut item = item.map_err(|e| archive_error(e.to_string()))?;
                // unpack_in refuses paths that would land outside `dest`.
                let inside = item
                    .unpack_in(dest)
                    .map_err(|e| archive_error(e.to_string()))?;
                if !inside {
                    let path = item
                        .path()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default();
                    return Err(archive_error(format!("güvensiz yol: {path}")));
                }
            }
        }
    }
    Ok(())
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<(), CatalogError> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path).at(path)?.permissions();
    perms.set_mode(perms.mode() | 0o755);
    fs::set_permissions(path, perms).at(path)
}

#[cfg(not(unix))]
fn make_executable(_: &Path) -> Result<(), CatalogError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::releases::test_server::serve;
    use mts_core::release::PlatformRelease;
    use mts_core::{Arch, Os};
    use sha2::{Digest, Sha256};
    use std::collections::{BTreeMap, HashMap};
    use std::io::Write;
    use std::sync::Mutex;

    const LINUX: Platform = Platform::new(Os::Linux, Arch::X64);

    fn release(
        version: &str,
        asset: &str,
        format: ArchiveFormat,
        entry: &str,
        bytes: &[u8],
    ) -> ReleaseManifest {
        ReleaseManifest {
            schema: 1,
            tool: "demo".into(),
            repo: "o/demo".into(),
            version: version.into(),
            tag: format!("v{version}"),
            commit: "c".into(),
            created_at: "x".into(),
            readme: "README.md".into(),
            platforms: BTreeMap::from([(
                "linux-x64".into(),
                PlatformRelease {
                    portable: PortableAsset {
                        asset: asset.into(),
                        sha256: hex::encode(Sha256::digest(bytes)),
                        size: bytes.len() as u64,
                        format,
                        entry: entry.into(),
                    },
                    installers: vec![],
                },
            )]),
        }
    }

    fn zip_bytes(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut buf = io::Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut buf);
            for (name, data) in files {
                zip.start_file(*name, zip::write::SimpleFileOptions::default())
                    .unwrap();
                zip.write_all(data).unwrap();
            }
            zip.finish().unwrap();
        }
        buf.into_inner()
    }

    fn tar_gz_bytes(files: &[(&str, &[u8])]) -> Vec<u8> {
        let gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        let mut tar = tar::Builder::new(gz);
        for (name, data) in files {
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append_data(&mut header, name, *data).unwrap();
        }
        tar.into_inner().unwrap().finish().unwrap()
    }

    fn write(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
        let p = dir.join(name);
        fs::write(&p, bytes).unwrap();
        p
    }

    #[test]
    fn installs_a_zip_and_lists_it() {
        let dl = tempfile::tempdir().unwrap();
        let tools = tempfile::tempdir().unwrap();
        let bytes = zip_bytes(&[("demo.exe", b"MZ"), ("data/readme.txt", b"hi")]);
        let r = release(
            "1.0.0",
            "demo-1.0.0-linux-x64.zip",
            ArchiveFormat::Zip,
            "demo.exe",
            &bytes,
        );
        let installer = Installer::new(tools.path());

        let installed = installer
            .install_downloaded(&r, LINUX, &write(dl.path(), "a.zip", &bytes))
            .unwrap();
        assert!(installed.entry.is_file());
        assert!(installed.folder.join("data/readme.txt").is_file());
        assert_eq!(installer.installed("demo"), vec![installed]);
        // Nothing left behind but the version folder.
        let names: Vec<_> = fs::read_dir(tools.path().join("demo"))
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        assert_eq!(names, ["1.0.0"]);
    }

    #[cfg(unix)]
    #[test]
    fn single_file_releases_become_executable() {
        use std::os::unix::fs::PermissionsExt;
        let dl = tempfile::tempdir().unwrap();
        let tools = tempfile::tempdir().unwrap();
        let name = "demo-1.0.0-linux-x64.AppImage";
        let r = release("1.0.0", name, ArchiveFormat::File, name, b"ELF");
        let installed = Installer::new(tools.path())
            .install_downloaded(&r, LINUX, &write(dl.path(), "x", b"ELF"))
            .unwrap();
        let mode = fs::metadata(&installed.entry).unwrap().permissions().mode();
        assert_eq!(mode & 0o111, 0o111);
    }

    #[test]
    fn installs_a_tar_gz_app_bundle() {
        let dl = tempfile::tempdir().unwrap();
        let tools = tempfile::tempdir().unwrap();
        let bytes = tar_gz_bytes(&[
            ("demo.app/Contents/Info.plist", b"<plist/>"),
            ("demo.app/Contents/MacOS/demo", b"bin"),
        ]);
        let r = release(
            "2.0.0",
            "demo.app.tar.gz",
            ArchiveFormat::TarGz,
            "demo.app/Contents/MacOS/demo",
            &bytes,
        );
        let installed = Installer::new(tools.path())
            .install_downloaded(&r, LINUX, &write(dl.path(), "a.tgz", &bytes))
            .unwrap();
        assert!(installed.entry.ends_with("demo.app/Contents/MacOS/demo"));
        assert!(installed.entry.is_file());
    }

    #[test]
    fn rejects_tampered_downloads_and_leaves_nothing() {
        let dl = tempfile::tempdir().unwrap();
        let tools = tempfile::tempdir().unwrap();
        let bytes = zip_bytes(&[("demo.exe", b"MZ")]);
        let r = release("1.0.0", "demo.zip", ArchiveFormat::Zip, "demo.exe", &bytes);
        let mut tampered = bytes.clone();
        *tampered.last_mut().unwrap() ^= 0xff;
        let installer = Installer::new(tools.path());
        let err = installer
            .install_downloaded(&r, LINUX, &write(dl.path(), "a.zip", &tampered))
            .unwrap_err();
        assert!(matches!(err, CatalogError::ShaMismatch { .. }), "{err}");
        assert!(installer.installed("demo").is_empty());
    }

    #[test]
    fn rejects_zip_slip() {
        let dl = tempfile::tempdir().unwrap();
        let tools = tempfile::tempdir().unwrap();
        let bytes = zip_bytes(&[("../../evil.txt", b"x"), ("demo.exe", b"MZ")]);
        let r = release("1.0.0", "demo.zip", ArchiveFormat::Zip, "demo.exe", &bytes);
        let installer = Installer::new(tools.path().join("tools"));
        let err = installer
            .install_downloaded(&r, LINUX, &write(dl.path(), "a.zip", &bytes))
            .unwrap_err();
        assert!(err.to_string().contains("güvensiz yol"), "{err}");
        assert!(!tools.path().join("evil.txt").exists());
        assert!(installer.installed("demo").is_empty());
        assert!(!tools.path().join("tools/demo/.partial-1.0.0").exists());
    }

    #[test]
    fn missing_entry_is_an_error() {
        let dl = tempfile::tempdir().unwrap();
        let tools = tempfile::tempdir().unwrap();
        let bytes = zip_bytes(&[("other.exe", b"MZ")]);
        let r = release("1.0.0", "demo.zip", ArchiveFormat::Zip, "demo.exe", &bytes);
        let err = Installer::new(tools.path())
            .install_downloaded(&r, LINUX, &write(dl.path(), "a.zip", &bytes))
            .unwrap_err();
        assert!(matches!(err, CatalogError::EntryMissing(_)));
    }

    #[test]
    fn no_build_for_platform() {
        let tools = tempfile::tempdir().unwrap();
        let r = release("1.0.0", "demo.zip", ArchiveFormat::Zip, "demo.exe", b"x");
        let err = Installer::new(tools.path())
            .install_downloaded(
                &r,
                Platform::new(Os::Windows, Arch::Arm64),
                Path::new("unused"),
            )
            .unwrap_err();
        assert!(matches!(err, CatalogError::NoBuildForPlatform { .. }));
    }

    #[test]
    fn folders_without_marker_are_not_installs_and_versions_sort_newest_first() {
        let dl = tempfile::tempdir().unwrap();
        let tools = tempfile::tempdir().unwrap();
        let installer = Installer::new(tools.path());
        for v in ["0.9.0", "0.10.0", "0.2.0"] {
            let bytes = zip_bytes(&[("demo.exe", v.as_bytes())]);
            let r = release(v, "demo.zip", ArchiveFormat::Zip, "demo.exe", &bytes);
            installer
                .install_downloaded(&r, LINUX, &write(dl.path(), v, &bytes))
                .unwrap();
        }
        fs::create_dir_all(tools.path().join("demo/1.0.0")).unwrap(); // no marker
        fs::create_dir_all(tools.path().join("demo/.partial-3.0.0")).unwrap();
        let versions: Vec<_> = installer
            .installed("demo")
            .into_iter()
            .map(|i| i.record.version)
            .collect();
        assert_eq!(versions, ["0.10.0", "0.9.0", "0.2.0"]);

        installer.uninstall("demo", "0.9.0").unwrap();
        assert_eq!(installer.installed("demo").len(), 2);
        assert!(matches!(
            installer.uninstall("demo", "0.9.0"),
            Err(CatalogError::NotInstalled { .. })
        ));
    }

    #[tokio::test]
    async fn end_to_end_download_install_and_idempotent_reinstall() {
        let bytes = zip_bytes(&[("demo.exe", b"MZ")]);
        let r = release(
            "1.0.0",
            "demo-1.0.0-linux-x64.zip",
            ArchiveFormat::Zip,
            "demo.exe",
            &bytes,
        );
        let server = serve(HashMap::from([(
            "/o/demo/releases/download/v1.0.0/demo-1.0.0-linux-x64.zip".into(),
            bytes.clone(),
        )]));
        let client = ReleaseClient::with_base(&server.base, "test").unwrap();
        let tools = tempfile::tempdir().unwrap();
        let installer = Installer::new(tools.path());
        let steps = Mutex::new(Vec::new());

        let first = installer
            .install(&client, &r, LINUX, &|s| steps.lock().unwrap().push(s))
            .await
            .unwrap();
        assert!(first.entry.is_file());
        let steps = steps.into_inner().unwrap();
        assert!(matches!(
            steps.first(),
            Some(Step::Downloading { received: 0, .. })
        ));
        assert_eq!(
            &steps[steps.len() - 2..],
            [Step::Verifying, Step::Unpacking]
        );

        // Second call: already installed, no download (the server would still
        // answer, so check the marker did not change).
        let again = installer
            .install(&client, &r, LINUX, &|_| {})
            .await
            .unwrap();
        assert_eq!(again.record.installed_at, first.record.installed_at);
    }

    #[tokio::test]
    async fn failed_download_cleans_up() {
        let bytes = zip_bytes(&[("demo.exe", b"MZ")]);
        let r = release("1.0.0", "demo.zip", ArchiveFormat::Zip, "demo.exe", &bytes);
        let server = serve(HashMap::new()); // 404 for everything
        let client = ReleaseClient::with_base(&server.base, "test").unwrap();
        let tools = tempfile::tempdir().unwrap();
        let installer = Installer::new(tools.path());
        assert!(
            installer
                .install(&client, &r, LINUX, &|_| {})
                .await
                .is_err()
        );
        assert!(!tools.path().join("demo/.partial-1.0.0").exists());
    }
}
