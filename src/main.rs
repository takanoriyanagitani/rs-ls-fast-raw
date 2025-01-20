use std::io;
use std::process::ExitCode;

#[cfg(any(doc, target_os = "macos"))]
fn sub_darwin() -> Result<(), io::Error> {
    rs_ls_fast_raw::ls::darwin::arg2dir2dirents2stdout()
}

fn sub() -> Result<(), io::Error> {
    #[cfg(any(doc, target_os = "macos"))]
    sub_darwin()?;

    Ok(())
}

fn main() -> ExitCode {
    sub().map(|_| ExitCode::SUCCESS).unwrap_or_else(|e| {
        eprintln!("{e}");
        ExitCode::FAILURE
    })
}
