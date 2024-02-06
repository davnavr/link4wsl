#![deny(unreachable_pub)]
#![deny(unsafe_code)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::cast_possible_truncation)]

macro_rules! write_err_ln {
    ($fmt:expr $(, $($arg:tt)*)?) => {{
        use std::io::Write as _;
        let _ = writeln!(std::io::stderr(), "LINK4WSL : {}", format_args!($fmt $(, $($arg)*)?));
    }};
}

macro_rules! fail {
    ($($arg:tt)*) => {{
        write_err_ln!($($arg)*);
        std::process::exit(-1)
    }};
}

macro_rules! env_vars {
    ($($name:ident,)*) => {
        mod env {
            $(pub(super) const $name: &str = stringify!($name);)*
        }
    };
}

env_vars!(LINK4WSL_PATH, LINK4WSL_DISTRO, LINK4WSL_LIB_DIRS,);

/// Translates a Linux-style `path` into a Windows-style path, and appends it to the given
/// `buffer`.
fn translate_path(buffer: &mut String, distro: &str, path: &str) {
    buffer.push_str(r"\\wsl.localhost\");
    buffer.push_str(distro);
    buffer.push('\\');
    for part in path.split_inclusive('/') {
        if let Some(portion) = part.strip_suffix('/') {
            buffer.push_str(portion);
            buffer.push('\\');
        } else {
            buffer.push_str(part)
        }
    }
}

fn main() -> ! {
    #[cfg(not(target_os = "linux"))]
    compile_error!("link4wsl must be built for a Linux environment");

    let mut arguments = std::env::args();

    // Skip program name
    let _ = arguments.next();

    if arguments.len() == 0 {
        // LINK.EXE by default lists all arguments with an annoying NEWLINE needed to skip them
        write_err_ln!("no arguments passed to LINK.EXE");
        std::process::exit(0)
    }

    let linker_path =
        std::path::PathBuf::from(std::env::var_os(env::LINK4WSL_PATH).unwrap_or_else(|| {
            fail!(
                "expected linux-style path to LINK.EXE in the {:?} environment variable",
                env::LINK4WSL_PATH
            )
        }));

    let distro = std::env::var(env::LINK4WSL_DISTRO).unwrap_or_else(|_| {
        fail!(
            "expected WSL Distro name in the {:?} environment variable",
            env::LINK4WSL_DISTRO
        )
    });

    let mut link = std::process::Command::new(&linker_path);

    // Iterator over all arguments, translating paths if necessary
    let mut buffer = String::new();
    let mut out_file = None;
    for arg in arguments {
        let mut actual_arg = &arg;

        if let Some(arg_start) = arg.strip_prefix('/') {
            if arg_start.contains('/') {
                use std::fmt::Write as _;

                // argument is most likely a path or contains a path
                buffer.clear();

                // Figure out if its /FLAG:PATH or PATH
                let (flag, path) = arg_start.split_once(':').unwrap_or(("", arg_start));

                if flag == "OUT" {
                    // If /OUT is specified, store the path to the resulting EXE to update its Linux
                    // permissions later.
                    out_file = Some(Box::<str>::from(path));
                }

                if !flag.is_empty() {
                    // Colon separates flag and path
                    let _ = write!(&mut buffer, "/{}:", flag);
                };

                // Append the translated path to the buffer
                translate_path(&mut buffer, &distro, path);

                actual_arg = &buffer;
            }
        }

        link.arg(actual_arg);
    }

    let _ = buffer;
    let out_file = out_file;

    link.env_clear();

    // Tell LINK.EXE where stuff like kernel32.lib and msvcrt.lib are
    match std::env::var(env::LINK4WSL_LIB_DIRS) {
        Ok(lib_directories) => {
            // Workaround to tell WSL to keep the LIB variable on the Windows side
            link.env("WSLENV", "LIB/w").env("LIB", lib_directories);
        }
        Err(std::env::VarError::NotPresent) => (),
        Err(std::env::VarError::NotUnicode(_)) => fail!("bad {:?} value", env::LINK4WSL_LIB_DIRS),
    };

    let mut link_process = link
        .spawn()
        .unwrap_or_else(|err| fail!("could not spawn {linker_path:?}: {err}"));

    let exit_code = link_process
        .wait()
        .unwrap_or_else(|err| fail!("LINK.EXE did not exit: {err}"))
        .code()
        .unwrap_or_else(|| fail!("LINK.EXE terminated by signal"));

    // TODO: Windows exit code is truncated. Instead, return -1 if anything is written to stderr
    if exit_code != 0 {
        write_err_ln!("linker invocation failed (exited with truncated code {exit_code:#02X})");
    } else if let Some(out_path) = out_file {
        // TODO: Figure out if a non-zero windows exit code could be truncated to a zero

        // If link was successful, mark any resulting EXE file as executable
        match std::fs::metadata(&*out_path) {
            Ok(metadata) => {
                use std::os::unix::fs::PermissionsExt as _;

                let mut perm = metadata.permissions();

                // Set the executable bits.
                perm.set_mode(0o777);

                if let Err(err) = std::fs::set_permissions(&*out_path, perm) {
                    fail!("could not mark output file {out_path:?} as executable: {err}");
                }
            }
            // Link was not successful, don't attempt to mark the file as executable
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => (),
            Err(err) => fail!("could not obtain permissions for output file {out_path:?}: {err}"),
        }
    }

    if exit_code == 1181 & 0xFF {
        write_err_ln!(
            concat!(
                "if libraries could not be found, try adding the contents of the LIB environment",
                "variable in your MSVC command prompt to {}"
            ),
            env::LINK4WSL_LIB_DIRS
        );
    }

    std::process::exit(exit_code)
}
