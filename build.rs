use std::{env, fs, path::Path};
use toml::Table;

static ZSTD_PATH: &str = "/opt/homebrew/Cellar/zstd/1.5.6/lib";

fn main() {
    if cfg!(target_os = "macos") {
        let current_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let cargo_toml_path = Path::new(&current_dir).join("Cargo.toml");
        let cargo_toml_content = fs::read_to_string(cargo_toml_path).unwrap();
        let value = cargo_toml_content.parse::<Table>().unwrap();

        if let toml::Value::String(feature) = &value["dependencies"]["inkwell"]["features"][0] {
            let (llvm_path, llvm_prefix) = match feature.as_str() {
                "llvm12-0" => ("LLVM_SYS_120_PREFIX", "/opt/homebrew/opt/llvm@12/"),
                "llvm13-0" => ("LLVM_SYS_130_PREFIX", "/opt/homebrew/opt/llvm@13/"),
                "llvm14-0" => ("LLVM_SYS_140_PREFIX", "/opt/homebrew/opt/llvm@14/"),
                "llvm15-0" => ("LLVM_SYS_150_PREFIX", "/opt/homebrew/opt/llvm@15/"),
                "llvm16-0" => ("LLVM_SYS_160_PREFIX", "/opt/homebrew/opt/llvm@16/"),
                "llvm17-0" => ("LLVM_SYS_170_PREFIX", "/opt/homebrew/opt/llvm@17/"),
                "llvm18-0" => ("LLVM_SYS_180_PREFIX", "/opt/homebrew/opt/llvm@18/"),
                "llvm18-1" => ("LLVM_SYS_181_PREFIX", "/opt/homebrew/opt/llvm@18/"),
                _ => panic!("No supported LLVM version found!"),
            };

            println!("cargo:rustc-env=LIBRARY_PATH={ZSTD_PATH}");
            println!("cargo:rustc-env={llvm_prefix}={llvm_path}");
            env::set_var("PATH", format!("{llvm_path}:$PATH"));
        }
    }
}
