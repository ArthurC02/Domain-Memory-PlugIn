//! The launcher in `bin/` and the release installers, run as a consumer runs them.

#[allow(dead_code, unused_imports)]
#[path = "../src/sha256.rs"]
mod sha256;

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ASSET: &str = if cfg!(windows) {
    "domain-memory-windows-x86_64.exe"
} else if cfg!(target_os = "macos") {
    "domain-memory-macos-universal"
} else {
    "domain-memory-linux-x86_64"
};
const INSTALLED: &str = if cfg!(windows) {
    "domain-memory.exe"
} else {
    "domain-memory"
};

fn plugin() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn scratch(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("dm-install-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

/// Serves `root` over HTTP on a local port until the test process ends.
fn serve(root: PathBuf) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut request = String::new();
            let mut reader = BufReader::new(&stream);
            if reader.read_line(&mut request).is_err() {
                continue;
            }
            loop {
                let mut header = String::new();
                if reader.read_line(&mut header).map_or(true, |read| read <= 2) {
                    break;
                }
            }
            let name = request
                .split(' ')
                .nth(1)
                .unwrap_or("/")
                .trim_start_matches('/');
            let mut stream = &stream;
            let _ = match std::fs::read(root.join(name)) {
                Ok(body) => stream
                    .write_all(
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            body.len()
                        )
                        .as_bytes(),
                    )
                    .and_then(|()| stream.write_all(&body)),
                Err(_) => stream.write_all(
                    b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                ),
            };
        }
    });
    address
}

fn release(name: &str, sums_digest: Option<&str>) -> (String, PathBuf) {
    let root = scratch(name);
    let dist = root.join("dist");
    std::fs::create_dir_all(&dist).unwrap();
    let binary = std::fs::read(env!("CARGO_BIN_EXE_domain-memory")).unwrap();
    std::fs::write(dist.join(ASSET), &binary).unwrap();
    let digest = sums_digest.map_or_else(|| sha256::hex_digest(&binary), str::to_string);
    std::fs::write(dist.join("SHA256SUMS"), format!("{digest}  {ASSET}\n")).unwrap();
    (serve(dist), root.join("bin"))
}

fn install_sh(base: &str, to: &Path) -> Output {
    Command::new("sh")
        .arg(plugin().join("install.sh"))
        .args(["--version", "v0.0.0-test", "--to"])
        .arg(to)
        .env("DOMAIN_MEMORY_DOWNLOAD_URL", base)
        .output()
        .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn the_launcher_runs_the_executable_committed_for_this_platform() {
    let run = Command::new("sh")
        .arg(plugin().join("bin").join("domain-memory"))
        .arg("-h")
        .output()
        .unwrap();
    assert!(run.status.success(), "{}", text(&run.stderr));
    assert!(
        text(&run.stdout).starts_with("usage: domain-memory"),
        "{}",
        text(&run.stdout)
    );
}

#[cfg(windows)]
#[test]
fn the_windows_launcher_runs_the_committed_executable() {
    let run = Command::new("cmd")
        .arg("/c")
        .arg(plugin().join("bin").join("domain-memory.cmd"))
        .arg("-h")
        .output()
        .unwrap();
    assert!(run.status.success(), "{}", text(&run.stderr));
    assert!(
        text(&run.stdout).starts_with("usage: domain-memory"),
        "{}",
        text(&run.stdout)
    );
}

#[test]
fn install_sh_installs_a_release_whose_digest_matches() {
    let (base, to) = release("sh-good", None);
    let run = install_sh(&base, &to);
    assert!(run.status.success(), "{}", text(&run.stderr));
    let installed = to.join(INSTALLED);
    assert!(
        text(&run.stdout).contains(&format!("Installed domain-memory {} at ", plugin_version())),
        "{}",
        text(&run.stdout)
    );
    let help = Command::new(&installed).arg("-h").output().unwrap();
    assert!(help.status.success());
}

#[test]
fn install_sh_refuses_a_release_whose_digest_differs_and_installs_nothing() {
    let (base, to) = release("sh-bad", Some(&"0".repeat(64)));
    let run = install_sh(&base, &to);
    assert_eq!(Some(1), run.status.code(), "{}", text(&run.stdout));
    assert!(
        text(&run.stderr).contains(&format!(
            "{ASSET} from {base} does not match its SHA256SUMS entry"
        )),
        "{}",
        text(&run.stderr)
    );
    assert!(!to.join(INSTALLED).exists());
}

#[cfg(windows)]
fn install_ps1(base: &str, to: &Path) -> Output {
    Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(plugin().join("install.ps1"))
        .args(["-Version", "v0.0.0-test", "-To"])
        .arg(to)
        .env("DOMAIN_MEMORY_DOWNLOAD_URL", base)
        .env("DOMAIN_MEMORY_NO_MODIFY_PATH", "1")
        .output()
        .unwrap()
}

#[cfg(windows)]
#[test]
fn install_ps1_installs_a_release_whose_digest_matches() {
    let (base, to) = release("ps1-good", None);
    let run = install_ps1(&base, &to);
    assert!(
        run.status.success(),
        "{}{}",
        text(&run.stdout),
        text(&run.stderr)
    );
    let help = Command::new(to.join(INSTALLED)).arg("-h").output().unwrap();
    assert!(help.status.success());
}

#[cfg(windows)]
#[test]
fn install_ps1_refuses_a_release_whose_digest_differs_and_installs_nothing() {
    let (base, to) = release("ps1-bad", Some(&"0".repeat(64)));
    let run = install_ps1(&base, &to);
    assert!(!run.status.success());
    assert!(
        text(&run.stderr).contains("does not match its SHA256SUMS entry"),
        "{}",
        text(&run.stderr)
    );
    assert!(!to.join(INSTALLED).exists());
}

fn plugin_version() -> String {
    let manifest =
        std::fs::read_to_string(plugin().join(".claude-plugin").join("plugin.json")).unwrap();
    let after = manifest.split("\"version\": \"").nth(1).unwrap();
    after[..after.find('"').unwrap()].to_string()
}

#[test]
fn install_sh_takes_its_directory_from_the_environment() {
    let (base, to) = release("sh-env", None);
    let run = Command::new("sh")
        .arg(plugin().join("install.sh"))
        .env("DOMAIN_MEMORY_DOWNLOAD_URL", &base)
        .env("DOMAIN_MEMORY_INSTALL_DIR", &to)
        .output()
        .unwrap();
    assert!(run.status.success(), "{}", text(&run.stderr));
    assert!(to.join(INSTALLED).is_file());
    assert!(
        text(&run.stdout).contains("is not on PATH; add it"),
        "{}",
        text(&run.stdout)
    );
}

#[test]
fn install_sh_names_a_missing_release_and_reads_a_bare_number_as_its_tag() {
    let root = scratch("sh-missing");
    std::fs::create_dir_all(root.join("dist")).unwrap();
    let base = serve(root.join("dist"));
    let run = Command::new("sh")
        .arg(plugin().join("install.sh"))
        .args(["--version", "9.9.9", "--to"])
        .arg(root.join("bin"))
        .env("DOMAIN_MEMORY_DOWNLOAD_URL", &base)
        .output()
        .unwrap();
    assert_eq!(Some(1), run.status.code(), "{}", text(&run.stdout));
    assert!(
        text(&run.stderr).contains(&format!(
            "could not download {ASSET} from {base}; is v9.9.9 a release? The releases are listed at "
        )),
        "{}",
        text(&run.stderr)
    );
}

#[test]
fn install_sh_refuses_sums_other_than_the_pinned_ones() {
    let (base, to) = release("sh-pinned", None);
    let run = Command::new("sh")
        .arg(plugin().join("install.sh"))
        .args(["--version", "v0.0.0-test", "--to"])
        .arg(&to)
        .env("DOMAIN_MEMORY_DOWNLOAD_URL", &base)
        .env("DOMAIN_MEMORY_SHA256SUMS_SHA256", "0".repeat(64))
        .output()
        .unwrap();
    assert_eq!(Some(1), run.status.code(), "{}", text(&run.stdout));
    assert!(
        text(&run.stderr).contains("is not the one DOMAIN_MEMORY_SHA256SUMS_SHA256 pins"),
        "{}",
        text(&run.stderr)
    );
    assert!(!to.join(INSTALLED).exists());
}

#[cfg(windows)]
#[test]
fn install_ps1_names_a_missing_release_and_refuses_unpinned_sums() {
    let root = scratch("ps1-missing");
    std::fs::create_dir_all(root.join("dist")).unwrap();
    let missing = install_ps1(&serve(root.join("dist")), &root.join("bin"));
    assert!(!missing.status.success());
    assert!(
        text(&missing.stderr).contains("is v0.0.0-test a release?"),
        "{}",
        text(&missing.stderr)
    );
    let (base, to) = release("ps1-pinned", None);
    let pinned = Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(plugin().join("install.ps1"))
        .args(["-Version", "0.0.0-test", "-To"])
        .arg(&to)
        .env("DOMAIN_MEMORY_DOWNLOAD_URL", &base)
        .env("DOMAIN_MEMORY_NO_MODIFY_PATH", "1")
        .env("DOMAIN_MEMORY_SHA256SUMS_SHA256", "0".repeat(64))
        .output()
        .unwrap();
    assert!(!pinned.status.success());
    assert!(
        text(&pinned.stderr).contains("is not the one DOMAIN_MEMORY_SHA256SUMS_SHA256 pins"),
        "{}",
        text(&pinned.stderr)
    );
    assert!(!to.join(INSTALLED).exists());
}
