use std::io::Error;

type Result<T> = core::result::Result<T, Error>;

pub(super) fn install_service() -> Result<()> {
    use std::ffi::OsString;

    Ok(())
}
