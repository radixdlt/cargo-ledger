use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::process::Command;

pub fn retrieve_data_size(file: &std::path::Path) -> Result<u64, io::Error> {
    let buffer = fs::read(file)?;
    let elf = goblin::elf::Elf::parse(&buffer).unwrap();

    let mut nvram_data = 0;
    let mut envram_data = 0;
    for s in elf.syms.iter() {
        let symbol_name = elf.strtab.get(s.st_name);
        let name = symbol_name.unwrap().unwrap();
        match name {
            "_nvram_data" => nvram_data = s.st_value,
            "_envram_data" => envram_data = s.st_value,
            _ => (),
        }
    }
    Ok(envram_data - nvram_data)
}

pub fn export_binary(elf_path: &std::path::Path, dest_bin: &std::path::Path) {
    let objcopy = env::var_os("CARGO_TARGET_THUMBV6M_NONE_EABI_OBJCOPY")
        .unwrap_or_else(|| "arm-none-eabi-objcopy".into());

    Command::new(objcopy)
        .arg(elf_path)
        .arg(dest_bin)
        .args(["-O", "ihex"])
        .output()
        .expect("Objcopy failed");

    let size = env::var_os("CARGO_TARGET_THUMBV6M_NONE_EABI_SIZE")
        .unwrap_or_else(|| "arm-none-eabi-size".into());

    // print some size info while we're here
    let out = Command::new(size)
        .arg(elf_path)
        .output()
        .expect("Size failed");

    io::stdout().write_all(&out.stdout).unwrap();
    io::stderr().write_all(&out.stderr).unwrap();
}

pub fn install_with_ledgerctl(
    is_dry_run: bool,
    use_python: bool,
    dir: &std::path::Path,
    app_json: &std::path::Path,
) {
    let mut arguments = Vec::new();

    arguments.extend_from_slice(if use_python {
        &["-m", "ledgerctl", "install", "-f"]
    } else {
        &["install", "-f"]
    });
    arguments.push(app_json.to_str().unwrap());

    let mut command =
        Command::new(if use_python { "python3" } else { "ledgerctl" });

    if !is_dry_run {
        command.current_dir(dir);
    }

    command.args(arguments);

    if is_dry_run {
        println!("{:?}", command);
    } else {
        let out = command.output().expect("fail");

        io::stdout().write_all(&out.stdout).unwrap();
        io::stderr().write_all(&out.stderr).unwrap();
    }
}
