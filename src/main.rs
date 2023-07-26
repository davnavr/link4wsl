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

const LINKER_ENV_VAR: &str = "LINK4WSL_PATH";

fn main() -> ! {
    let mut arguments = std::env::args_os();

    // Skip program name
    let _ = arguments.next();

    if arguments.len() == 0 {
        // LINK.EXE by default lists all arguments with an annoying NEWLINE needed to skip them
        let _ = write_err_ln!("no arguments passed to LINK.EXE");
        std::process::exit(0)
    }

    let linker_path = std::path::PathBuf::from(
        std::env::var_os(LINKER_ENV_VAR)
            .unwrap_or_else(|| fail!("expected LINK.EXE path in {LINKER_ENV_VAR:?}")),
    );

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

    let _ = write_err_ln!("exited with code {exit_code}");

    std::process::exit(exit_code)
}
