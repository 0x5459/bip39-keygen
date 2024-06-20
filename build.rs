use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let build_gitcommit_fallback = "Unknown (no git or not git repo)".to_string();

    vergen::EmitBuilder::builder()
        .build_timestamp()
        .all_git()
        .emit()?;

    let git_hash = Command::new("git")
        .args(["describe", "--always", "--match=NeVeRmAtCh", "--dirty"])
        .output()
        .map_err(|e| e.to_string())
        .and_then(|output| String::from_utf8(output.stdout).map_err(|e| e.to_string()))
        .unwrap_or(build_gitcommit_fallback);

    println!("cargo:rustc-env=GIT_COMMIT={}", git_hash);

    Ok(())
}
