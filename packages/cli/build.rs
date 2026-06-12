// build.rs — warn if rosettax87_jit CMake binaries are stale or missing.
fn main() {
    let script =
        env!("CARGO_MANIFEST_DIR").replace("\\", "/") + "/../../scripts/check-rosetta-freshness.sh";

    let output = std::process::Command::new("bash").arg(&script).output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);

            // Emit cargo:warning= lines so they appear in build output.
            for line in stdout.lines() {
                if !line.is_empty() {
                    println!("cargo:warning={}", line);
                }
            }
            for line in stderr.lines() {
                if !line.is_empty() {
                    eprintln!("{}", line);
                }
            }

            // Re-run this script when the binaries themselves change.
            let build_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../vendor/rosettax87_jit/build/bin");
            if build_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&build_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            println!("cargo:rerun-if-changed={}", path.display());
                        }
                    }
                }
            }
        }
        Err(e) => {
            // Script not found — not a warning, just silently skip.
            eprintln!("check-rosetta-freshness.sh not found at {}: {}", script, e);
        }
    }
}
