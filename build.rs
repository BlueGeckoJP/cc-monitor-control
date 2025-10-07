use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=web-ui/src");
    println!("cargo:rerun-if-changed=web-ui/package.json");
    println!("cargo:rerun-if-changed=web-ui/vite.config.ts");

    // Run `bun run build` in the `web-ui` directory
    let status = Command::new("bun")
        .args(["run", "build"])
        .current_dir("web-ui")
        .status()
        .expect("Failed to execute bun run build");

    if !status.success() {
        panic!("bun run build failed with status: {}", status);
    }

    println!("Successfully built web-ui");
}
