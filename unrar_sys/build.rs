use std::env;

fn main() {
    // A build script is compiled for the host, so `cfg!(windows)` here answers for the machine
    // doing the building rather than the one being built for. Cross compiling reads the wrong
    // answer out of it: a Windows host building for Android compiled isnt.cpp and stopped at
    // wbemidl.h, and a Linux host building for Windows left isnt.cpp out of a library that
    // needs it. `CARGO_CFG_TARGET_*` is what a build script has to ask instead.
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let windows = target_os == "windows";
    if windows {
        println!("cargo:rustc-flags=-lpowrprof");
        println!("cargo:rustc-link-lib=shell32");
        // pathfn.cpp reads the registry to find the RAR data directory and system.cpp asks
        // about the process token; both are here. Without it an MSVC link of a crate that
        // depends on this one ends in unresolved RegOpenKeyExW and OpenProcessToken.
        println!("cargo:rustc-link-lib=advapi32");
        if target_env == "gnu" {
            println!("cargo:rustc-link-lib=pthread");
        }
    } else if target_os != "android" {
        // Android keeps the pthread functions in libc and ships no libpthread to link against.
        println!("cargo:rustc-link-lib=pthread");
    }
    let mut names = vec![
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
    ];
    if windows {
        names.push("isnt");
    }
    let files: Vec<String> = names.iter().map(|&s| format!("vendor/unrar/{s}.cpp")).collect();
    let mut build = cc::Build::new();
    build
        .cpp(true) // Switch to C++ library compilation.
        .opt_level(2)
        .std("c++14")
        // by default cc crate tries to link against dynamic stdlib, which causes problems on windows-gnu target
        .cpp_link_stdlib(None)
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
    if target_os == "android" {
        // Nothing links the C++ runtime otherwise: the NDK does not link one by default and
        // `cpp_link_stdlib(None)` above, which is there for windows-gnu, says not to ask for
        // one. Static, so that a crate depending on this one is not left to ship
        // libc++_shared.so beside it.
        build.cpp_link_stdlib(Some("c++_static"));
        // libc++_static leaves the ABI symbols to this one, so a link without it ends in
        // "undefined symbol: operator new" and "vtable for std::logic_error".
        println!("cargo:rustc-link-lib=c++abi");
        // ulinks.cpp restores a symlink's timestamps with lutimes, which bionic does not have.
        // os.hpp only reaches for lutimes where it thinks there is no utimensat, which it
        // decides from _POSIX_C_SOURCE; Android has utimensat and does not set that. Saying so
        // takes the utimensat path, which is both the one that builds here and the precise one.
        build.define("UNIX_TIME_NS", None);
    }
    build.files(&files).compile("libunrar.a");
}
