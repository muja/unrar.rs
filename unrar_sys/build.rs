fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if cfg!(windows) {
        println!("cargo:rustc-flags=-lpowrprof");
        println!("cargo:rustc-link-lib=shell32");
        if cfg!(target_env = "gnu") {
            println!("cargo:rustc-link-lib=pthread");
        }
    } else if target_os != "android" {
        // Android's Bionic libc includes pthread; no separate libpthread exists
        println!("cargo:rustc-link-lib=pthread");
    }

    // iOS/tvOS/watchOS require explicit C++ stdlib linking
    let is_apple_mobile = target_os == "ios" || target_os == "tvos" || target_os == "watchos";
    if is_apple_mobile {
        println!("cargo:rustc-link-lib=c++");
    }

    let files: Vec<String> = [
        "strlist",
        "strfn",
        "pathfn",
        "smallfn",
        "global",
        "file",
        "filefn",
        "filcreat",
        "archive",
        "arcread",
        "unicode",
        "system",
        #[cfg(windows)]
        "isnt",
        "crypt",
        "crc",
        "rawread",
        "encname",
        "match",
        "timefn",
        "rdwrfn",
        "consio",
        "options",
        "errhnd",
        "rarvm",
        "secpassword",
        "rijndael",
        "getbits",
        "sha1",
        "sha256",
        "blake2s",
        "hash",
        "extinfo",
        "extract",
        "volume",
        "list",
        "find",
        "unpack",
        "headers",
        "threadpool",
        "rs16",
        "cmddata",
        "ui",
        "filestr",
        "scantree",
        "dll",
        "qopen",
    ]
    .iter()
    .map(|&s| format!("vendor/unrar/{s}.cpp"))
    .collect();

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .opt_level(2)
        .std("c++14")
        .warnings(false)
        .extra_warnings(false)
        .flag_if_supported("-stdlib=libc++")
        .flag_if_supported("-fPIC")
        .flag_if_supported("-Wno-switch")
        .flag_if_supported("-Wno-parentheses")
        .flag_if_supported("-Wno-macro-redefined")
        .flag_if_supported("-Wno-dangling-else")
        .flag_if_supported("-Wno-logical-op-parentheses")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-variable")
        .flag_if_supported("-Wno-unused-function")
        .flag_if_supported("-Wno-missing-braces")
        .flag_if_supported("-Wno-unknown-pragmas")
        .flag_if_supported("-Wno-deprecated-declarations")
        .define("_FILE_OFFSET_BITS", Some("64"))
        .define("_LARGEFILE_SOURCE", None)
        .define("RAR_SMP", None)
        .define("RARDLL", None);

    // Only disable cpp_link_stdlib on Windows GNU where it causes issues
    // On Apple platforms, we need libc++ linked
    if target.contains("windows") && target.contains("gnu") {
        build.cpp_link_stdlib(None);
    } else if is_apple_mobile {
        build.cpp_link_stdlib(Some("c++"));
    }

    // Set deployment target for Apple mobile platforms
    if is_apple_mobile {
        let min_version = match target_os.as_str() {
            "ios" => "12.0",
            "tvos" => "12.0",
            "watchos" => "5.0",
            _ => "12.0",
        };

        if target.contains("-sim") || target.contains("x86_64") {
            // Simulator
            let flag = match target_os.as_str() {
                "ios" => format!("-mios-simulator-version-min={}", min_version),
                "tvos" => format!("-mtvos-simulator-version-min={}", min_version),
                "watchos" => format!("-mwatchos-simulator-version-min={}", min_version),
                _ => format!("-mios-simulator-version-min={}", min_version),
            };
            build.flag(&flag);
        } else if target.contains("-macabi") {
            // Mac Catalyst - target triple encodes the version
            build.flag("-target");
            build.flag(&format!(
                "{}-apple-ios13.1-macabi",
                if target.contains("x86_64") {
                    "x86_64"
                } else {
                    "arm64"
                }
            ));
        } else {
            // Device
            let flag = match target_os.as_str() {
                "ios" => format!("-mios-version-min={}", min_version),
                "tvos" => format!("-mtvos-version-min={}", min_version),
                "watchos" => format!("-mwatchos-version-min={}", min_version),
                _ => format!("-mios-version-min={}", min_version),
            };
            build.flag(&flag);
        }
    }

    build.files(&files).compile("libunrar.a");
}
