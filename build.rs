extern crate syntex;
extern crate serde_codegen;

use std::env;
use std::path::Path;
use std::fs;

#[derive(PartialEq)]
pub enum PlatformKind {
    Windows,
    Unix,
    Other,
}


pub fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    let src = Path::new("src/weather_structs.in.rs");
    let dst = Path::new(&out_dir).join("weather_structs.rs");

    let mut registry = syntex::Registry::new();

    serde_codegen::register(&mut registry);
    registry.expand("", &src, &dst).unwrap();

    // bundle in libeay32.dll and ssleay32.dll if we're on windows
    if get_platform_kind() == PlatformKind::Windows {
        println!("Build: Detected Windows, copying libeay32.dll and ssleay32.dll...");
        if let Some(mingw_paths) = get_mingw_in_path() {
            for path in mingw_paths {
                copy_mingw_lib_to_exe_dir(&path, "libeay32.dll");
                copy_mingw_lib_to_exe_dir(&path, "ssleay32.dll");
            }
        }
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn copy_mingw_lib_to_exe_dir(mingw_path: &str, filename: &str) {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let exe_dir = Path::new(out_dir.to_str().unwrap()) //Go up to the dir the .exe will live in (i.e. target -> [debug|release])
                      .parent().unwrap()
                      .parent().unwrap()
                      .parent().unwrap();

    let src = Path::new(mingw_path).join(filename);
    let dest = Path::new(&exe_dir).join(filename);

    println!("Copying {} from {:?} to {:?}", filename, &src, &dest);
    match fs::copy(&src, &dest) {
        Err(e) =>  println!("Failed to copy {} from {:?} to {:?}!\nErrorKind: {:?}",filename, src, dest, e),        
        Ok(_) => println!("Successfully copied {}", filename)
    }
}

#[cfg(target_family="windows")]
fn get_platform_kind() -> PlatformKind {
    return PlatformKind::Windows;
}

#[cfg(target_family="unix")]
fn get_platform_kind() -> PlatformKind {
    return PlatformKind::Unix;
}

#[cfg(not(any(target_family="windows", target_family="unix")))]
fn get_platform_kind() -> PlatformKind {
    return PlatformKind::Other;
}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn get_mingw_in_path() -> Option<Vec<String>> {
    match env::var_os("PATH") {
        Some(env_path) => {
            let paths: Vec<String> = env::split_paths(&env_path).filter_map(|path| {
                use std::ascii::AsciiExt;

                match path.to_str() {
                    Some(path_str) => {
                        if path_str.to_ascii_lowercase().contains("mingw") {
                            Some(path_str.to_string())
                        } else { None }
                    },
                    None => None
                }
            }).collect();

            if paths.len() > 0 { 
                Some(paths) 
            } else { 
                None 
            }
        },
        None => None
    }
}