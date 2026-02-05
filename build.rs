use std::path::Path;
use std::process::Command;

fn main() {
    let templates_dir = "templates";

    // Only clone if the templates directory doesn't exist
    if !Path::new(templates_dir).exists() {
        let status = Command::new("git")
            .args([
                "clone",
                "--depth=1",
                "https://github.com/github/gitignore.git",
                templates_dir,
            ])
            .status()
            .expect("Failed to execute git clone");

        if !status.success() {
            panic!("Failed to clone gitignore templates repository");
        }

        // Remove the .git directory to save space and avoid nested repo issues
        let git_dir = Path::new(templates_dir).join(".git");
        if git_dir.exists() {
            std::fs::remove_dir_all(&git_dir).expect("Failed to remove .git directory");
        }
    }

    // Tell Cargo to rerun build.rs if it changes
    println!("cargo:rerun-if-changed=build.rs");
}
