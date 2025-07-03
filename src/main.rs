mod parser;
mod core;

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

use std::process::Command;
use std::env;

use core::APP_VERSION;
use core::LATEST_DATE;
use parser::parse_file;
use core::eval_all;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 && args[1] == "--version" {
        println!("Trion Performance X {}", APP_VERSION);
        return;
    }

    if args.len() == 2 && args[1] == "--info" {
        println!("Trion Performance X");
        println!("----------------------------------");
        println!("Version : {}", APP_VERSION);
        println!("Author : ARF Developer");
        println!("First release date : 7/1/2025");
        println!("Latest release date : {}", LATEST_DATE);
        return;
    }

    if args.len() == 2 && args[1] == "--help" {
        println!("Usage: triax <script.tpx>");
        println!("Options:\n  --version   Print version info\n  --help      Show this message");
        println!("All triax command:");
        println!("--help");
        println!("--version");
        println!("--info");
        println!("--run [your file.tpx]");
        println!("--uninstall");
        return;
    }

    if args.len() == 3 && args[1] == "--run" {
        let path = &args[2];
        let stmts = parse_file(path);
        eval_all(&stmts, 3);
        return;
    }

    #[cfg(windows)]
    if args.len() == 2 && args[1] == "--uninstall" {
        println!("⚠️ Trion is uninstalling...");

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let trion = hkcu.open_subkey("Software\\Trion");
        let uninstall_path = match trion {
            Ok(key) => match key.get_value::<String, _>("") {
                Ok(path) => format!(r#"{}\UninstallTrion.exe"#, path),
                Err(_) => {
                    eprintln!("❌ Install path not found in registry.");
                    return;
                }
            },
            Err(_) => {
                eprintln!("❌ Registry key not found.");
                return;
            }
        };

        match Command::new(&uninstall_path).spawn() {
            Ok(_) => println!("🧼 Uninstaller launched from: {}", uninstall_path),
            Err(e) => eprintln!("❌ Failed to launch uninstaller: {}", e),
        }
        return;
    }

    #[cfg(not(windows))]
    if args.len() == 2 && args[1] == "--uninstall" {
        println!("❌ Uninstall is only supported on Windows.");
        return;
    }

    if args[1].starts_with("--") {
        eprintln!("⚠️  Unrecognized flag: {}", args[1]);
        eprintln!("Use --help to see available options.");
        std::process::exit(1);
    }

    if args.len() != 2 {
        eprintln!("Usage: triax <script.tpx>");
        std::process::exit(1);
    }

    let path = &args[1];
    let stmts = parse_file(path);
    eval_all(&stmts, 3); // warning threshold = 3
}
