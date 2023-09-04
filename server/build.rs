use std::fs;
use std::process::Command;

fn main() {
    let output = Command::new("wasm-pack")
        .args(&[
            "build",
            "--target",
            "web",
            "../client",
            "--no-pack",
            "--no-typescript",
        ])
        .output()
        .expect("Unable to call wasm-pack");

    if !output.status.success() {
        panic!(
            "Error while compiling:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Even if the build succeeded, we wanted the WARNINGS
    println!("{}", String::from_utf8_lossy(&output.stderr));

    for file_name in [
        "pixel_demolition_client_bg.wasm",
        "pixel_demolition_client.js",
    ] {
        fs::copy(
            format!("../client/pkg/{file_name}"),
            format!("static/{file_name}"),
        )
        .expect("Unable to copy generated files");
    }
}
