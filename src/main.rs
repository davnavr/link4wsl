//! Entry point for `link4wsl`.

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

env_vars!(LINK4WSL_PATH, LINK4WSL_DISTRO,);

// "\\wsl.localhost\{DISTRO}\{}"

fn main() -> ! {
    let mut arguments = std::env::args_os();

    // Skip program name
    let _ = arguments.next();

    if arguments.len() == 0 {
        // LINK.EXE by default lists all arguments with an annoying NEWLINE needed to skip them
        write_err_ln!("no arguments passed to LINK.EXE");
        std::process::exit(0)
    }

    let linker_path = std::path::PathBuf::from(
        std::env::var_os(env::LINK4WSL_PATH)
            .unwrap_or_else(|| fail!("expected LINK.EXE path in {:?}", env::LINK4WSL_PATH)),
    );

    let distro = std::env::var_os(env::LINK4WSL_DISTRO)
        .unwrap_or_else(|| fail!("expected WSL Distro name in {:?}", env::LINK4WSL_DISTRO));

    let cleaned_arguments = arguments; // TODO: Clean up args

    let mut link_process = std::process::Command::new(&linker_path)
        .args(cleaned_arguments)
        .spawn()
        .unwrap_or_else(|err| fail!("could not spawn {linker_path:?}: {err}"));

    let exit_code = link_process
        .wait()
        .unwrap_or_else(|err| fail!("LINK.EXE did not exit: {err}"))
        .code()
        .unwrap_or_else(|| fail!("LINK.EXE terminated by signal"));

    write_err_ln!("exited with code {exit_code}");

    std::process::exit(exit_code)
}
