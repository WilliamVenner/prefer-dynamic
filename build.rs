#[macro_use]
extern crate build_cfg;

use std::{borrow::Cow, ffi::OsStr, path::PathBuf};

#[build_cfg_main]
fn main() {
	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-env-changed=OUT_DIR");
	println!("cargo:rerun-if-env-changed=RUSTUP_HOME");
	println!("cargo:rerun-if-env-changed=RUSTUP_TOOLCHAIN");

	let target_dir = PathBuf::from(std::env::var("OUT_DIR").expect("Expected OUT_DIR env var to bet set"));

	let target_dir = if target_dir.join("deps").is_dir() {
		Cow::Owned(target_dir)
	} else {
		Cow::Borrowed(
			target_dir
				.parent().unwrap()
				.parent().unwrap()
				.parent().unwrap(),
		)
	};

	let home = std::env::var("RUSTUP_HOME").expect("Expected RUSTUP_HOME env var to bet set");
	let toolchain = std::env::var("RUSTUP_TOOLCHAIN").expect("Expected RUSTUP_TOOLCHAIN env var to bet set");

	let lib_ext = OsStr::new(if build_cfg!(target_os = "windows") {
		"dll"
	} else if build_cfg!(target_os = "macos") {
		"dylib"
	} else {
		"so"
	});

	let mut lib_path = PathBuf::from(&home);
	lib_path.push("toolchains");
	lib_path.push(toolchain);

	let mut found = false;
	for lib_path in [lib_path.join("lib"), lib_path.join("bin")] {
		if !lib_path.is_dir() {
			continue;
		}
		for lib in lib_path
			.read_dir()
			.expect("Failed to read toolchain directory")
			.map(|entry| entry.expect("Failed to read toolchain directory entry"))
			.filter_map(|entry| {
				if entry
					.file_type()
					.expect("Failed to read toolchain directory entry type")
					.is_file()
				{
					Some(entry.path())
				} else {
					None
				}
			})
			.filter(|path| path.extension() == Some(lib_ext))
		{
			if let Some(os_file_name) = lib.file_name() {
				let file_name = os_file_name.to_string_lossy();
				let file_name = file_name.strip_prefix("lib").unwrap_or_else(|| file_name.as_ref());
				if file_name.starts_with("std-") {
					found = true;
					std::fs::copy(&lib, target_dir.join(os_file_name))
						.expect("Failed to copy std lib to target directory");
				} else if cfg!(feature = "link-test") && file_name.starts_with("test-") {
					found = true;
					std::fs::copy(&lib, target_dir.join(os_file_name))
						.expect("Failed to copy test lib to target directory");
				}
			}
		}
	}

	if !found {
		panic!("Failed to find std lib in toolchain directory!");
	}
}
