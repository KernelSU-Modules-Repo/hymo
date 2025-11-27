mod zip_ext;

use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use fs_extra::dir;
use zip::{write::FileOptions, CompressionMethod};

use crate::zip_ext::zip_create_from_directory_with_options;

#[derive(Parser)]
#[command(name = "xtask")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the full project (zakosign disabled temporarily)
    Build {
        /// Build in release mode
        #[arg(long)]
        release: bool,
        /// (Disabled) Path to signing private key
        #[arg(long)]
        sign_key: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = project_root();

    match cli.command {
        Commands::Build { release, sign_key: _ } => {
            let output_dir = root.join("output");
            let module_build_dir = output_dir.join("module_files");

            // 1. Clean & Setup
            println!(":: Cleaning output directory...");
            if output_dir.exists() {
                fs::remove_dir_all(&output_dir)?;
            }
            fs::create_dir_all(&module_build_dir)?;

            // 2. Build WebUI
            build_webui(&root)?;

            // 3. Build Zakosign (Host Tool) - [DISABLED]
            // println!(":: Building Zakosign (Host)...");
            // let zakosign_bin = build_zakosign(&root)?;
            let zakosign_bin: Option<PathBuf> = None; 

            // 4. Build Core (Android)
            let core_bin = build_core(&root, release)?;

            // 5. Copy Module Files
            println!(":: Copying module files...");
            let module_src = root.join("module");
            dir::copy(
                &module_src,
                &output_dir,
                &dir::CopyOptions::new().overwrite(true).content_only(true),
            )?;
            
            // Cleanup gitignore
            let gitignore = module_build_dir.join(".gitignore");
            if gitignore.exists() { fs::remove_file(gitignore)?; }

            // 6. Inject Version
            let version = inject_version(&module_build_dir)?;
            fs::write(output_dir.join("version"), &version)?;

            // 7. Install Core Binary
            let dest_bin = module_build_dir.join("meta-hybrid");
            fs::copy(&core_bin, &dest_bin)?;

            // [DISABLED] Signing Logic
            /*
            if let Some(zakosign) = zakosign_bin {
                if let Some(key) = resolve_sign_key(sign_key) {
                    println!(":: Signing meta-hybrid binary...");
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Ok(metadata) = fs::metadata(&zakosign) {
                            let mut perms = metadata.permissions();
                            perms.set_mode(0o755);
                            let _ = fs::set_permissions(&zakosign, perms);
                        }
                    }

                    let status = Command::new(zakosign)
                        .current_dir(&root)
                        .arg("sign")
                        .arg(&dest_bin)
                        .arg("--key")
                        .arg(key)
                        .arg("--output")
                        .arg(&dest_bin)
                        .arg("--force")
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .status()
                        .context("Failed to execute zakosign")?;

                    if !status.success() {
                        anyhow::bail!("Zakosign signing failed!");
                    }
                    println!(":: Binary signed successfully.");
                } else {
                    println!(":: [WARNING] No signing key found (ZAKOSIGN_KEY or --sign-key). Skipping signature.");
                }
            }
            */

            // 8. Zip Package
            println!(":: Creating zip archive...");
            let options = FileOptions::default()
                .compression_method(CompressionMethod::Deflated)
                .compression_level(Some(9));
            
            let zip_name = format!("meta-hybrid-{}.zip", version);
            let zip_path = output_dir.join(zip_name);
            
            zip_create_from_directory_with_options(
                &zip_path,
                &module_build_dir,
                |_| options,
            )?;

            println!(":: Build success: {}", zip_path.display());
        }
    }

    Ok(())
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

fn build_webui(root: &Path) -> Result<()> {
    println!(":: Building WebUI...");
    let webui_dir = root.join("webui");
    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };

    let status = Command::new(npm)
        .current_dir(&webui_dir)
        .arg("install")
        .status()?;
    if !status.success() { anyhow::bail!("npm install failed"); }

    let status = Command::new(npm)
        .current_dir(&webui_dir)
        .args(["run", "build"])
        .status()?;
    if !status.success() { anyhow::bail!("npm run build failed"); }

    Ok(())
}

/* [DISABLED]
fn build_zakosign(root: &Path) -> Result<Option<PathBuf>> {
    let zakosign_dir = root.join("zakosign");
    if !zakosign_dir.exists() {
        return Ok(None);
    }

    println!(":: Building Zakosign (Host)...");
    let setup_script = zakosign_dir.join("tools/setupdep");
    
    if setup_script.exists() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(&setup_script) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o755);
                let _ = fs::set_permissions(&setup_script, perms);
            }
        }

        // 1. Compile dependencies (Host Only)
        let status_build = Command::new(&setup_script)
            .current_dir(&zakosign_dir)
            .env_remove("ANDROID_NDK_HOME")
            .env_remove("ANDROID_NDK_ROOT")
            .env_remove("ANDROID_NDK")
            .stdout(Stdio::inherit())
            .stderr(
            