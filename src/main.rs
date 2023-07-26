use std::path::PathBuf;

macro_rules! fail {
    ($fmt:expr $(, $($arg:tt)*)?) => {{
        use std::io::Write as _;
        let _ = writeln!(std::io::stderr(), "LINK4WSL : {}", format_args!($fmt $(, $($arg)*)?));
        std::process::exit(-1)
    }};
}

fn main() -> ! {
    let mut arguments = std::env::args_os();

    let linker_path = PathBuf::from(
        arguments
            .next()
            .unwrap_or_else(|| fail!("expected link.exe path as the first argument")),
    );

    let cleaned_arguments = arguments; // TODO: Clean up args

    let mut link_process = std::process::Command::new(&linker_path)
        .args(cleaned_arguments)
        .spawn()
        .unwrap_or_else(|err| fail!("could not spawn {linker_path:?}: {err}"));

    let exit_code = link_process
        .wait()
        .unwrap_or_else(|err| fail!("linker process did not exit: {err}"))
        .code()
        .unwrap_or_else(|| fail!("linker process terminated by signal"));

    std::process::exit(exit_code)
}
