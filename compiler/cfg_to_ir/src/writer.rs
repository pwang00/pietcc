use inkwell::module::Module;
use std::fs::{remove_file, OpenOptions};
use std::io::{Error, Write};
use std::process::Command;

pub(crate) fn generate_executable(module: &Module, filename: &str) -> Result<(), Error> {
    let bitcode_fname = &format!("{}.bc", filename);
    let asm_fname = &format!("{}.s", filename);

    let _bitcode_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(bitcode_fname)?;

    let _asm_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(asm_fname)?;

    let bitcode_path = std::path::Path::new(bitcode_fname);
    module.write_bitcode_to_path(bitcode_path);

    let llc_output = Command::new("llc")
        .arg(bitcode_fname)
        .arg("-o")
        .arg(asm_fname)
        .output()
        .expect("Failed to execute command");

    if !llc_output.status.success() {
        let stderr = String::from_utf8_lossy(&llc_output.stderr);
        eprintln!("Error: {}", stderr);
    }

    let clang_output = Command::new("clang")
        .arg("-no-pie")
        .arg(asm_fname)
        .arg("-o")
        .arg(filename)
        .output()
        .expect("Failed to execute command");

    if !clang_output.status.success() {
        let stderr = String::from_utf8_lossy(&llc_output.stderr);
        eprintln!("Error: {}", stderr);
    }

    remove_file(bitcode_fname)?;
    remove_file(asm_fname)?;
    Ok(())
}

pub(crate) fn generate_llvm_bitcode(module: &Module, filename: &str) -> Result<(), Error> {
    let bitcode_file_name = &format!("{}.bc", filename);
    let _bitcode_file = OpenOptions::new()
        .write(true) // Enable writing to the file
        .create(true) // Create the file if it doesn't exist
        .open(bitcode_file_name)?;

    let bitcode_path = std::path::Path::new(bitcode_file_name);
    module.write_bitcode_to_path(bitcode_path);
    Ok(())
}

pub(crate) fn generate_llvm_ir(module: &Module, filename: &str) -> Result<(), Error> {
    let llvm_ir_file_name = &format!("{}.ll", filename);
    let llvm_ir_path = std::path::Path::new(llvm_ir_file_name);
    let mut llvm_ir_file = OpenOptions::new()
        .write(true) // Enable writing to the file
        .create(true) // Create the file if it doesn't exist
        .open(llvm_ir_path)?;

    llvm_ir_file.write_all(module.print_to_string().to_bytes())?;
    Ok(())
}
