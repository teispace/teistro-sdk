//! `check-package`'s Java arms (`03-design/java-binding.md` §9): the staged
//! jar used as a consumer uses it, on the module path with the JDK alone
//! and on the class path as Maven resolves it from the staged repository.
//!
//! The teeth are the library line. `target/dist/check` is inside the
//! checkout, so a loader that fell through the jar's own library to the
//! workspace's build would still answer; the consumer prints the path it
//! loaded and the gate requires it under the cache it was given.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::binding::{java_command, present, step, tool};
use crate::hashes::hex;
use crate::java_binding::{MODULE, javac};
use crate::java_package::{ARTIFACT, GROUP, MAVEN, version_dir};
use crate::platform::Platform;
use crate::rel;
use crate::zip;

/// The consumer, which knows nothing but the published names.
const CONSUMER: &str = "bindings/java/packaging/Consumer.java";
/// The dependency plugin, pinned on the command line: Maven resolves and
/// nothing else, so no compiler or jar plugin enters.
const COPY_DEPENDENCIES: &str =
    "org.apache.maven.plugins:maven-dependency-plugin:3.11.0:copy-dependencies";

/// Both arms, in `check/java`.
pub(crate) fn check(
    root: &Path,
    dist: &Path,
    check: &Path,
    platform: &Platform,
    version: &str,
) -> Result<(), ()> {
    if !(present("javac", "--version") && present("java", "--version")) {
        crate::skip::skip("the Java package: no javac and java on this machine");
        return Ok(());
    }
    let into = check.join("java");
    let layout = dist.join(MAVEN);
    let jars = version_dir(&layout, version);
    let default = jars.join(format!("{ARTIFACT}-{version}.jar"));
    let classifier = jars.join(format!("{ARTIFACT}-{version}-{}.jar", platform.name()));
    if !default.is_file() || !classifier.is_file() {
        println!(
            "FAIL  no Java package was staged for {} under {}",
            platform.name(),
            rel(root, &jars)
        );
        return Err(());
    }
    let module = module_path(root, dist, &into, platform, version, &default, &classifier);
    let maven = maven(
        root,
        &into,
        platform,
        version,
        &layout,
        &default,
        &classifier,
    );
    module.and(maven)
}

// ── (a) the module path ───────────────────────────────────────────────────

/// The jar on the module path, the JDK alone and no network.
fn module_path(
    root: &Path,
    dist: &Path,
    into: &Path,
    platform: &Platform,
    version: &str,
    default: &Path,
    classifier: &Path,
) -> Result<(), ()> {
    let described = output(
        Command::new("jar")
            .arg("--describe-module")
            .arg("--file")
            .arg(default),
        "the jar does not describe its module",
    )?;
    let first = described.lines().next().unwrap_or_default();
    if !first.starts_with(&format!("{MODULE}@{version}")) {
        println!("FAIL  the jar's module reads `{first}`, not {MODULE}@{version}");
        return Err(());
    }
    println!("ok    the jar is the module {MODULE}@{version}");
    entries(root, dist, default)?;

    let cache = into.join("cache");
    let arm = Arm::module(default);
    let library = run_consumer(root, into, &arm, &cache)?;
    let written = modified(&library)?;
    let again = run_consumer(root, into, &arm, &cache)?;
    if again != library || modified(&library)? != written {
        println!("FAIL  a second run did not reuse {}", rel(root, &library));
        return Err(());
    }
    println!("ok    a second run reuses the cached library");

    // Red: a cached library that is not the jar's is refused, never loaded.
    // The loader leaves it read-only, so it is made writable first.
    fs::metadata(&library)
        .and_then(|meta| {
            let mut permissions = meta.permissions();
            #[allow(clippy::permissions_set_readonly_false)]
            permissions.set_readonly(false);
            fs::set_permissions(&library, permissions)
        })
        .and_then(|()| fs::write(&library, b"not the library"))
        .map_err(|err| println!("FAIL  cannot overwrite the cached library: {err}"))?;
    let tampered = consumer(into, &arm, &cache)
        .output()
        .map_err(|err| println!("FAIL  the consumer did not start: {err}"))?;
    let said = String::from_utf8_lossy(&tampered.stderr);
    if tampered.status.success() || !said.contains("hashes to") {
        println!("FAIL  a tampered cached library was not refused by its digest:\n{said}");
        return Err(());
    }
    println!("ok    a tampered cached library is refused by its digest");
    fs::remove_dir_all(&cache).map_err(|err| println!("FAIL  cannot empty the cache: {err}"))?;

    run_consumer(
        root,
        into,
        &Arm::module(classifier),
        &into.join("classifier-cache"),
    )?;
    println!(
        "ok    the {} classifier jar loads its own library",
        platform.name()
    );
    Ok(())
}

/// How a consumer is compiled and run against a jar.
struct Arm {
    /// What the report calls it.
    what: &'static str,
    /// What `--enable-native-access` names.
    access: &'static str,
    /// The compiler's arguments.
    compile: Vec<String>,
    /// The launcher's, before the main class.
    run: Vec<String>,
}

impl Arm {
    /// The jar as a named module, the consumer on the class path beside it.
    fn module(jar: &Path) -> Self {
        let module = vec![
            "--module-path".to_owned(),
            jar.display().to_string(),
            "--add-modules".to_owned(),
            MODULE.to_owned(),
        ];
        let mut run = module.clone();
        run.extend(["--class-path".to_owned(), "classes".to_owned()]);
        Self {
            what: "module path",
            access: MODULE,
            compile: module,
            run,
        }
    }

    /// The jar and the consumer both on the class path, the unnamed module.
    fn class_path(jar: &Path, windows: bool) -> Self {
        let separator = if windows { ";" } else { ":" };
        Self {
            what: "class path",
            access: "ALL-UNNAMED",
            compile: vec!["--class-path".to_owned(), jar.display().to_string()],
            run: vec![
                "--class-path".to_owned(),
                format!("{}{separator}classes", jar.display()),
            ],
        }
    }
}

/// The jar's entries against the merged manifest: the manifest first, the
/// natives exactly the staged platforms', the IDL the repository's.
fn entries(root: &Path, dist: &Path, jar: &Path) -> Result<(), ()> {
    let bytes = fs::read(jar).map_err(|err| println!("FAIL  cannot read the jar: {err}"))?;
    let names = zip::names(&bytes).map_err(|err| println!("FAIL  the jar: {err}"))?;
    let merged: Value = fs::read_to_string(dist.join("manifest.json"))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .ok_or_else(|| println!("FAIL  the merged manifest does not read"))?;
    let platforms: Vec<&str> = merged["maven"]["platforms"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let mut carried: Vec<&str> = names
        .iter()
        .filter_map(|name| name.strip_prefix("native/")?.split_once('/'))
        .map(|(platform, _)| platform)
        .collect();
    carried.dedup();
    let api = zip::read(&bytes, "META-INF/teistro/api.json")
        .map_err(|err| println!("FAIL  the jar: {err}"))?;
    let idl = fs::read(root.join("idl/api.json")).ok();
    if names.first().map(String::as_str) != Some("META-INF/MANIFEST.MF") {
        println!("FAIL  the jar's first entry is not its manifest");
        return Err(());
    }
    if carried != platforms {
        println!(
            "FAIL  the jar carries the libraries of {carried:?}, the manifest lists {platforms:?}"
        );
        return Err(());
    }
    if api.is_none() || api != idl {
        println!("FAIL  the jar's META-INF/teistro/api.json is not idl/api.json");
        return Err(());
    }
    println!(
        "ok    the jar holds its manifest first, the IDL and the libraries of {}",
        platforms.join(", ")
    );
    Ok(())
}

// ── (b) Maven ─────────────────────────────────────────────────────────────

/// The coordinate resolved by Maven from the staged repository, its
/// checksums enforced, and the consumer run on the class path.
fn maven(
    root: &Path,
    into: &Path,
    platform: &Platform,
    version: &str,
    layout: &Path,
    default: &Path,
    classifier: &Path,
) -> Result<(), ()> {
    let Some(mvn) = tool("mvn", "--version") else {
        if platform.libc == Some("musl") {
            crate::skip::excused(
                "the Java package through Maven: no `mvn` on this machine",
                "Maven reads the POM the same on every host; the musl question is the native's selection, which the module-path arm answers",
            );
        } else {
            crate::skip::skip("the Java package through Maven: no `mvn` on this machine");
        }
        return Ok(());
    };
    let maven = Maven {
        mvn,
        repository: root.join("target/tools/m2"),
        into,
        version,
    };
    for (project, staged, named) in [
        ("maven-default", default, None),
        ("maven-classifier", classifier, Some(platform.name())),
    ] {
        maven.consumed(root, platform, project, layout, staged, named.as_deref())?;
    }
    maven.checksums_enforced(layout)
}

/// Maven, resolving into a local repository of its own.
struct Maven<'a> {
    /// The launcher, as [`tool`] found it.
    mvn: String,
    /// The local repository, kept between runs so plugins are fetched once.
    repository: PathBuf,
    /// Where the projects are written.
    into: &'a Path,
    version: &'a str,
}

impl Maven<'_> {
    /// Resolves the coordinate in a project of its own from `from`, and
    /// answers its `lib`, or what Maven printed when it refused.
    fn resolve(
        &self,
        project: &str,
        from: &Path,
        classifier: Option<&str>,
    ) -> Result<PathBuf, String> {
        // The artefact always comes from the staged layout, never from a
        // previous run's local repository.
        let ours = self.repository.join(GROUP.replace('.', "/"));
        if ours.exists() {
            fs::remove_dir_all(&ours)
                .map_err(|err| format!("cannot empty {}: {err}", ours.display()))?;
        }
        let dir = self.into.join(project);
        crate::consumer::write(&dir.join("pom.xml"), &pom(self.version, from, classifier))
            .map_err(|()| "the project could not be written".to_owned())?;
        let out = Command::new(&self.mvn)
            .args(["-B", "-ntp", "-q"])
            .arg(format!("-Dmaven.repo.local={}", self.repository.display()))
            .arg(COPY_DEPENDENCIES)
            .arg("-DoutputDirectory=lib")
            .current_dir(&dir)
            .output()
            .map_err(|err| format!("`{}` did not start: {err}", self.mvn))?;
        if out.status.success() {
            Ok(dir.join("lib"))
        } else {
            Err(format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            ))
        }
    }

    /// One project: the staged jar, resolved alone and byte for byte, and
    /// the consumer run on the class path against it.
    fn consumed(
        &self,
        root: &Path,
        platform: &Platform,
        project: &str,
        layout: &Path,
        staged: &Path,
        classifier: Option<&str>,
    ) -> Result<(), ()> {
        let lib = self.resolve(project, layout, classifier).map_err(|said| {
            println!(
                "FAIL  Maven did not resolve {GROUP}:{ARTIFACT}:{}{} from the staged repository (it needs the network once, for its dependency plugin):\n{said}",
                self.version,
                classifier.map(|name| format!(":{name}")).unwrap_or_default()
            );
        })?;
        let file = staged.file_name().unwrap_or_default();
        let found: Vec<_> = fs::read_dir(&lib)
            .into_iter()
            .flatten()
            .flatten()
            .map(|entry| entry.file_name())
            .collect();
        if found != [file] || digest(&lib.join(file)) != digest(staged) {
            println!(
                "FAIL  Maven resolved {found:?}, not the staged {}",
                file.to_string_lossy()
            );
            return Err(());
        }
        let dir = self.into.join(project);
        run_consumer(
            root,
            &dir,
            &Arm::class_path(&lib.join(file), platform.is_windows()),
            &dir.join("cache"),
        )?;
        println!(
            "ok    {project}: Maven resolves the staged jar and it loads its library on the class path"
        );
        Ok(())
    }

    /// Red: one wrong byte in a checksum, and Maven refuses the artefact.
    fn checksums_enforced(&self, layout: &Path) -> Result<(), ()> {
        let version = self.version;
        let broken = self.into.join("maven-broken-layout");
        crate::package::copy_tree(layout, &broken)
            .map_err(|err| println!("FAIL  cannot copy the layout: {err}"))?;
        let sha1 = version_dir(&broken, version).join(format!("{ARTIFACT}-{version}.jar.sha1"));
        let text =
            fs::read_to_string(&sha1).map_err(|err| println!("FAIL  {}: {err}", sha1.display()))?;
        let flipped = if text.starts_with('0') { "1" } else { "0" };
        crate::consumer::write(
            &sha1,
            &format!("{flipped}{}", text.get(1..).unwrap_or_default()),
        )?;
        // Any refusal would pass a check that only asked for one, a machine
        // without the network among them: this one must name the checksum.
        match self.resolve("maven-broken", &broken, None) {
            Err(said) if said.contains("Checksum validation failed") => {
                println!("ok    Maven refuses a jar whose checksum is wrong");
                Ok(())
            }
            Err(said) => {
                println!(
                    "FAIL  Maven refused the broken layout, and not for its checksum:\n{said}"
                );
                Err(())
            }
            Ok(_) => {
                println!(
                    "FAIL  Maven resolved a jar whose checksum is wrong: the staged repository's checksums are not enforced"
                );
                Err(())
            }
        }
    }
}

/// A project depending on the coordinate from a repository on disk, its
/// checksums enforced.
fn pom(version: &str, repository: &Path, classifier: Option<&str>) -> String {
    // Forward slashes, also on Windows, and three after the scheme.
    let path = repository.display().to_string().replace('\\', "/");
    let url = if path.starts_with('/') {
        format!("file://{path}")
    } else {
        format!("file:///{path}")
    };
    let classifier = classifier
        .map(|name| format!("\n      <classifier>{name}</classifier>"))
        .unwrap_or_default();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
  <modelVersion>4.0.0</modelVersion>
  <groupId>check</groupId>
  <artifactId>consumer</artifactId>
  <version>0</version>
  <repositories>
    <repository>
      <id>staged</id>
      <url>{url}</url>
      <releases><checksumPolicy>fail</checksumPolicy></releases>
      <snapshots><enabled>false</enabled></snapshots>
    </repository>
  </repositories>
  <dependencies>
    <dependency>
      <groupId>{GROUP}</groupId>
      <artifactId>{ARTIFACT}</artifactId>
      <version>{version}</version>{classifier}
    </dependency>
  </dependencies>
</project>
"#
    )
}

// ── shared ────────────────────────────────────────────────────────────────

/// Compiles the consumer into `dir/classes` with `args` and runs it,
/// requiring the library it loaded to lie under `cache`; answers that
/// library.
fn run_consumer(root: &Path, dir: &Path, arm: &Arm, cache: &Path) -> Result<PathBuf, ()> {
    let what = arm.what;
    step(
        javac(&dir.join("classes"))
            .args(&arm.compile)
            .arg(root.join(CONSUMER)),
        "",
        &format!("the Java consumer did not compile against the {what} jar"),
    )?;
    let out = consumer(dir, arm, cache)
        .output()
        .map_err(|err| println!("FAIL  the Java consumer did not start: {err}"))?;
    let printed = String::from_utf8_lossy(&out.stdout);
    if !out.status.success() {
        println!(
            "FAIL  the Java consumer did not answer ({what}):\n{printed}{}",
            String::from_utf8_lossy(&out.stderr)
        );
        return Err(());
    }
    let library = printed
        .lines()
        .find_map(|line| line.strip_prefix("library "))
        .map(PathBuf::from)
        .ok_or_else(|| println!("FAIL  the Java consumer printed no library:\n{printed}"))?;
    let under = |path: &Path| fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if !under(&library).starts_with(under(cache)) {
        println!(
            "FAIL  the Java consumer ({what}) loaded {}, not the jar's library under {}",
            library.display(),
            rel(root, cache)
        );
        return Err(());
    }
    println!(
        "ok    the Java consumer ({what}) answers as the library does, from the jar's own library"
    );
    Ok(library)
}

/// The consumer's command: the jar's library found only by the jar.
fn consumer(dir: &Path, arm: &Arm, cache: &Path) -> Command {
    let mut command = java_command();
    command
        .arg(format!("--enable-native-access={}", arm.access))
        .args(&arm.run)
        .arg(format!("-D{CACHE_PROPERTY}={}", cache.display()))
        .arg("Consumer")
        .current_dir(dir)
        .env_remove("TEISTRO_LIBRARY");
    command
}

/// The system property the loader writes the jar's library under
/// (`Teistro.CACHE_PROPERTY`).
const CACHE_PROPERTY: &str = "teistro.cache";

/// A command's standard output, or a failure line.
fn output(command: &mut Command, failed: &str) -> Result<String, ()> {
    match command.output() {
        Ok(out) if out.status.success() => Ok(String::from_utf8_lossy(&out.stdout).into_owned()),
        Ok(out) => {
            println!("FAIL  {failed}:\n{}", String::from_utf8_lossy(&out.stderr));
            Err(())
        }
        Err(err) => {
            println!("FAIL  {failed}: it did not start: {err}");
            Err(())
        }
    }
}

/// When a file was last written.
fn modified(path: &Path) -> Result<std::time::SystemTime, ()> {
    fs::metadata(path)
        .and_then(|meta| meta.modified())
        .map_err(|err| println!("FAIL  {}: {err}", path.display()))
}

/// A file's SHA-256, or nothing when it does not read.
fn digest(path: &Path) -> Option<String> {
    fs::read(path).ok().map(|bytes| hex(&Sha256::digest(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_repository_url_has_three_slashes_on_every_host() {
        assert!(pom("1.0.0", Path::new("/a/maven"), None).contains("<url>file:///a/maven</url>"));
        assert!(
            pom("1.0.0", Path::new(r"C:\a\maven"), None).contains("<url>file:///C:/a/maven</url>")
        );
    }

    #[test]
    fn a_classifier_is_named_only_when_asked() {
        assert!(!pom("1.0.0", Path::new("/m"), None).contains("<classifier>"));
        assert!(
            pom("1.0.0", Path::new("/m"), Some("linux-x64"))
                .contains("<classifier>linux-x64</classifier>")
        );
        assert!(
            pom("1.0.0", Path::new("/m"), None).contains("<checksumPolicy>fail</checksumPolicy>")
        );
    }
}
