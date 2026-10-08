//! `cargo xtask publish maven [--automatic] [--dry-run] [--dist DIR]`: the
//! staged Maven layout (`java_package.rs`) signed, bundled and uploaded
//! to the Central Portal's publisher API.
//!
//! Nothing here builds: the layout is re-verified against the digests the
//! merged manifest recorded and its own checksum files, so what is signed
//! is what `package stage` wrote. The deployment is `USER_MANAGED` unless
//! `--automatic` is passed: Central validates it and it waits in the
//! Portal to be published by hand, which is how a first release, or a
//! rehearsal, is meant to go. `--dry-run` signs and bundles and uploads
//! nothing, and is the one form allowed at the unreleased version or on a
//! partial stage.
//!
//! The credentials are read from the environment and handed to `gpg` and
//! `curl` on their standard input, never on a command line, which other
//! processes can read: `MAVEN_CENTRAL_USERNAME` and
//! `MAVEN_CENTRAL_PASSWORD` (a Portal user token), `MAVEN_GPG_FINGERPRINT`
//! (the signing key, already imported) and `MAVEN_GPG_PASSPHRASE`.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::hashes::hex;
use crate::java_package::{ARTIFACT, CHECKSUMS, GROUP, MAVEN, checksums, version_dir};
use crate::platform::PLATFORMS;
use crate::release::{self, UNRELEASED};
use crate::zip::{self, Entry};

/// The Portal's publisher API.
const API: &str = "https://central.sonatype.com/api/v1/publisher";
/// How often a deployment's state is asked for.
const POLL: Duration = Duration::from_secs(10);
/// How long an automatic deployment is waited on before the command says
/// it is still publishing (which cannot then be stopped).
const PATIENCE: Duration = Duration::from_secs(60 * 60);

/// What the command was asked to do.
struct Options {
    automatic: bool,
    dry_run: bool,
    dist: PathBuf,
}

pub(crate) fn run(root: &Path, args: &[String]) -> i32 {
    let mut options = Options {
        automatic: false,
        dry_run: false,
        dist: root.join("target/dist"),
    };
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--automatic" => options.automatic = true,
            "--dry-run" => options.dry_run = true,
            "--dist" => {
                let Some(dir) = rest.next() else {
                    println!("FAIL  --dist names no directory");
                    return 2;
                };
                options.dist = PathBuf::from(dir);
            }
            other => {
                println!(
                    "FAIL  `publish maven` takes --automatic, --dry-run and --dist DIR, not {other}"
                );
                return 2;
            }
        }
    }
    match publish(root, &options) {
        Ok(()) => 0,
        Err(why) => {
            println!("FAIL  {why}");
            1
        }
    }
}

fn publish(root: &Path, options: &Options) -> Result<(), String> {
    let version = release::version(root);
    let manifest: Value = fs::read_to_string(options.dist.join("manifest.json"))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .ok_or("the merged manifest does not read: run `cargo xtask package stage` first")?;
    let staged: Vec<&str> = manifest["maven"]["platforms"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let every: Vec<String> = PLATFORMS
        .iter()
        .map(crate::platform::Platform::name)
        .collect();
    if let Some(why) = refusal(&version, &staged, &every, options.dry_run) {
        return Err(why);
    }
    let dir = version_dir(&options.dist.join(MAVEN), &version);
    verify(&options.dist, &dir, &manifest)?;
    let files = staged_files(&dir)?;
    for file in to_sign(&files) {
        sign(&dir.join(file))?;
    }
    let bundle = options
        .dist
        .join(format!("{ARTIFACT}-maven-bundle-{version}.zip"));
    let entries = bundle_entries(&dir, &version)?;
    fs::write(
        &bundle,
        zip::write(entries.iter().map(|(name, entry)| (name, entry)))
            .map_err(|err| format!("the bundle: {err}"))?,
    )
    .map_err(|err| format!("{}: {err}", bundle.display()))?;
    println!("wrote {} ({} files)", bundle.display(), entries.len());
    if options.dry_run {
        for (name, _) in &entries {
            println!("      {name}");
        }
        println!("a dry run: nothing was uploaded");
        return Ok(());
    }
    let header = format!(
        "Authorization: Bearer {}",
        bearer(
            &env("MAVEN_CENTRAL_USERNAME")?,
            &env("MAVEN_CENTRAL_PASSWORD")?
        )
    );
    let kind = if options.automatic {
        "AUTOMATIC"
    } else {
        "USER_MANAGED"
    };
    let id = curl(
        &header,
        &[
            "--form".to_owned(),
            format!("bundle=@{};type=application/octet-stream", bundle.display()),
            format!("{API}/upload?name={ARTIFACT}-{version}&publishingType={kind}"),
        ],
    )?
    .trim()
    .to_owned();
    println!("uploaded as deployment {id} ({kind})");
    wait(&header, &id, options.automatic)
}

/// Why this publish may not go ahead, if it may not: a real one needs a
/// released version and every platform's library in the jar.
fn refusal(version: &str, staged: &[&str], every: &[String], dry_run: bool) -> Option<String> {
    if dry_run {
        return None;
    }
    if version == UNRELEASED {
        return Some(format!(
            "{UNRELEASED} is the unreleased version: nothing is published from it (a --dry-run is)"
        ));
    }
    let missing: Vec<&str> = every
        .iter()
        .map(String::as_str)
        .filter(|name| !staged.contains(name))
        .collect();
    (!missing.is_empty()).then(|| {
        format!(
            "the staged jar lacks the libraries of {}: a partial stage is never published",
            missing.join(", ")
        )
    })
}

/// The staged files against the digests the manifest recorded, and every
/// checksum file against its file.
fn verify(dist: &Path, dir: &Path, manifest: &Value) -> Result<(), String> {
    let recorded = manifest["maven"]["files"]
        .as_array()
        .filter(|files| !files.is_empty())
        .ok_or("the merged manifest records no Maven files")?;
    for file in recorded {
        let name = file["file"].as_str().unwrap_or_default();
        let bytes = fs::read(dist.join(name)).map_err(|err| format!("{name}: {err}"))?;
        if Some(hex(&Sha256::digest(&bytes)).as_str()) != file["sha256"].as_str() {
            return Err(format!("{name} is not the file the stage recorded"));
        }
        let path = dist.join(name);
        for (extension, digest) in checksums(&bytes) {
            let beside = PathBuf::from(format!("{}.{extension}", path.display()));
            if fs::read_to_string(&beside).ok().as_deref() != Some(digest.as_str()) {
                return Err(format!(
                    "{} does not hold {name}'s digest",
                    beside.display()
                ));
            }
        }
    }
    let on_disk = staged_files(dir)?;
    let artefacts = to_sign(&on_disk).count();
    if artefacts != recorded.len() {
        return Err(format!(
            "{} holds {artefacts} artefacts and the manifest records {}",
            dir.display(),
            recorded.len()
        ));
    }
    println!("ok    the staged layout is the one the stage recorded");
    Ok(())
}

/// The files in the version's directory, by name, signatures left out
/// (a rerun signs afresh).
fn staged_files(dir: &Path) -> Result<Vec<String>, String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .map_err(|err| format!("{}: {err}", dir.display()))?
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| extension(name) != Some("asc"))
        .collect();
    names.sort();
    Ok(names)
}

/// The files Central wants signed: every one but the checksums, which it
/// wants unsigned.
fn to_sign(files: &[String]) -> impl Iterator<Item = &String> {
    files
        .iter()
        .filter(|name| !extension(name).is_some_and(|found| CHECKSUMS.contains(&found)))
}

/// A file name's extension, as the stage spells them (lower case).
fn extension(name: &str) -> Option<&str> {
    Path::new(name).extension().and_then(|found| found.to_str())
}

/// A detached armoured signature beside the file, checked as it is made.
fn sign(file: &Path) -> Result<(), String> {
    let key = env("MAVEN_GPG_FINGERPRINT")?;
    let passphrase = env("MAVEN_GPG_PASSPHRASE")?;
    let signature = PathBuf::from(format!("{}.asc", file.display()));
    piped(
        Command::new("gpg")
            .args(["--batch", "--yes", "--quiet", "--pinentry-mode", "loopback"])
            .args(["--passphrase-fd", "0", "--local-user", &key])
            .args(["--armor", "--detach-sign", "--output"])
            .arg(&signature)
            .arg(file),
        &passphrase,
        "gpg did not sign",
    )?;
    piped(
        Command::new("gpg")
            .args(["--batch", "--quiet", "--verify"])
            .arg(&signature)
            .arg(file),
        "",
        "gpg does not verify the signature it made",
    )?;
    Ok(())
}

/// The bundle's entries: the version's directory in the repository
/// layout, signatures included, by name.
fn bundle_entries(dir: &Path, version: &str) -> Result<Vec<(String, Entry)>, String> {
    let prefix = format!("{}/{ARTIFACT}/{version}", GROUP.replace('.', "/"));
    let mut names: Vec<String> = fs::read_dir(dir)
        .map_err(|err| format!("{}: {err}", dir.display()))?
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
        .into_iter()
        .map(|name| {
            let bytes = fs::read(dir.join(&name)).map_err(|err| format!("{name}: {err}"))?;
            Ok((format!("{prefix}/{name}"), Entry::Plain(bytes)))
        })
        .collect()
}

/// The Portal's bearer value for a user token.
fn bearer(username: &str, password: &str) -> String {
    STANDARD.encode(format!("{username}:{password}"))
}

/// A deployment's state, as the status call answers it.
#[derive(Debug, PartialEq, Eq)]
enum State {
    /// Still on its way, by the name the Portal gives.
    Pending(String),
    Validated,
    Published,
    /// Refused, with what the Portal said.
    Failed(String),
}

fn state(status: &Value) -> State {
    match status["deploymentState"].as_str().unwrap_or_default() {
        "VALIDATED" => State::Validated,
        "PUBLISHED" => State::Published,
        "FAILED" => State::Failed(status["errors"].to_string()),
        other => State::Pending(other.to_owned()),
    }
}

/// Polls until the deployment reaches the state this publish waits for:
/// validated for a user-managed one, published for an automatic one.
fn wait(header: &str, id: &str, automatic: bool) -> Result<(), String> {
    let started = Instant::now();
    loop {
        std::thread::sleep(POLL);
        let text = curl(
            header,
            &[
                "--request".to_owned(),
                "POST".to_owned(),
                format!("{API}/status?id={id}"),
            ],
        )?;
        let status: Value = serde_json::from_str(&text)
            .map_err(|err| format!("the status did not read: {err}: {text}"))?;
        match state(&status) {
            State::Failed(errors) => {
                return Err(format!(
                    "Central refused deployment {id} (kept in the Portal for support): {errors}"
                ));
            }
            State::Published => {
                println!("ok    deployment {id} is published");
                return Ok(());
            }
            State::Validated if !automatic => {
                println!("ok    deployment {id} is validated: publish or drop it in the Portal");
                return Ok(());
            }
            State::Validated | State::Pending(_) if started.elapsed() > PATIENCE => {
                println!(
                    "note  deployment {id} is still {}: it can no longer be stopped, and the Portal says when it lands",
                    status["deploymentState"]
                );
                return Ok(());
            }
            State::Validated | State::Pending(_) => {}
        }
    }
}

/// `curl` with the authorization header on its standard input; answers
/// the body.
fn curl(header: &str, args: &[String]) -> Result<String, String> {
    piped(
        Command::new("curl")
            .args([
                "--fail-with-body",
                "--silent",
                "--show-error",
                "--header",
                "@-",
            ])
            .args(args),
        header,
        "the Portal refused the call",
    )
}

/// Runs a command with `input` on its standard input; answers its
/// standard output.
fn piped(command: &mut Command, input: &str, failed: &str) -> Result<String, String> {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| {
            format!(
                "{failed}: `{}` did not start: {err}",
                command.get_program().to_string_lossy()
            )
        })?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .map_err(|err| format!("{failed}: {err}"))?;
    }
    let out = child
        .wait_with_output()
        .map_err(|err: io::Error| format!("{failed}: {err}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    if out.status.success() {
        Ok(stdout)
    } else {
        Err(format!(
            "{failed}: {stdout}{}",
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

/// A variable the publish needs.
fn env(name: &str) -> Result<String, String> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{name} is not set"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn every() -> Vec<String> {
        vec!["linux-x64".to_owned(), "darwin-arm64".to_owned()]
    }

    #[test]
    fn the_unreleased_version_and_a_partial_stage_are_refused() {
        let all = ["linux-x64", "darwin-arm64"];
        assert!(refusal(UNRELEASED, &all, &every(), false).is_some());
        let partial = refusal("1.0.0", &["linux-x64"], &every(), false).unwrap();
        assert!(partial.contains("darwin-arm64"), "{partial}");
        assert_eq!(refusal("1.0.0", &all, &every(), false), None);
        assert_eq!(refusal(UNRELEASED, &[], &every(), true), None, "a dry run");
    }

    #[test]
    fn every_file_but_a_checksum_is_signed() {
        let files: Vec<String> = [
            "teistro-1.0.0.jar",
            "teistro-1.0.0.jar.md5",
            "teistro-1.0.0.jar.sha1",
            "teistro-1.0.0.jar.sha256",
            "teistro-1.0.0.jar.sha512",
            "teistro-1.0.0.pom",
            "teistro-1.0.0.pom.sha1",
        ]
        .map(str::to_owned)
        .to_vec();
        assert_eq!(
            to_sign(&files).collect::<Vec<_>>(),
            ["teistro-1.0.0.jar", "teistro-1.0.0.pom"]
        );
    }

    #[test]
    fn the_bundle_is_the_repository_layout() {
        let dir = std::env::temp_dir().join(format!("teistro-bundle-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        for name in [
            "teistro-1.0.0.pom",
            "teistro-1.0.0.pom.asc",
            "teistro-1.0.0.pom.md5",
        ] {
            fs::write(dir.join(name), name).unwrap();
        }
        let names: Vec<String> = bundle_entries(&dir, "1.0.0")
            .unwrap()
            .into_iter()
            .map(|(name, _)| name)
            .collect();
        fs::remove_dir_all(&dir).unwrap();
        assert_eq!(
            names,
            [
                "com/teispace/teistro/1.0.0/teistro-1.0.0.pom",
                "com/teispace/teistro/1.0.0/teistro-1.0.0.pom.asc",
                "com/teispace/teistro/1.0.0/teistro-1.0.0.pom.md5",
            ]
        );
    }

    #[test]
    fn the_bearer_value_is_the_token_in_base64() {
        assert_eq!(bearer("user", "pass"), "dXNlcjpwYXNz");
    }

    /// The status example on Sonatype's publisher API page, and its failed
    /// twin.
    #[test]
    fn a_status_reads_as_its_state() {
        let validated = json!({
            "deploymentId": "28570f16-da32-4c14-bd2e-c1acc0782365",
            "deploymentName": "central-bundle.zip",
            "deploymentState": "VALIDATED",
            "purls": ["pkg:maven/com.sonatype.central.example/example_java_project@0.0.7"]
        });
        assert_eq!(state(&validated), State::Validated);
        assert_eq!(
            state(&json!({ "deploymentState": "PUBLISHING" })),
            State::Pending("PUBLISHING".to_owned())
        );
        let failed =
            state(&json!({ "deploymentState": "FAILED", "errors": { "pom": ["no url"] } }));
        assert!(
            matches!(failed, State::Failed(ref why) if why.contains("no url")),
            "{failed:?}"
        );
    }
}
