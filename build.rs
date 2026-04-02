use std::process::Command;

fn main() {
    let version = env!("CARGO_PKG_VERSION");

    let sha = run_git(&["rev-parse", "--short", "HEAD"]).unwrap_or_default();
    let date = run_git(&["log", "-1", "--format=%ci"])
        .map(|s| s.split_whitespace().next().unwrap_or("").to_string())
        .unwrap_or_default();

    println!("cargo:rustc-env=XFEAT_VERSION={version}");
    println!("cargo:rustc-env=XFEAT_GIT_SHA={sha}");
    println!("cargo:rustc-env=XFEAT_BUILT_AT={date}");
    println!("cargo:rerun-if-changed=.git/HEAD");
}

fn run_git(args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}
