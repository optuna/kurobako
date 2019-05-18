use crate::{Error, Result};
use std::io::Write as _;
use std::process::Command;
use std::sync::Arc;
use tempfile::{NamedTempFile, TempPath};

#[derive(Debug, Clone)]
pub struct EmbeddedScript {
    script_path: Arc<TempPath>,
}
impl EmbeddedScript {
    pub fn new(script: &str) -> Result<Self> {
        let mut temp_file = track!(NamedTempFile::new().map_err(Error::from))?;
        track!(write!(temp_file.as_file_mut(), "{}", script).map_err(Error::from))?;
        let script_path = temp_file.into_temp_path();

        #[cfg(unix)]
        {
            use std::fs;
            use std::os::unix::fs::PermissionsExt as _;

            track!(
                fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755))
                    .map_err(Error::from)
            )?;
        }

        Ok(Self {
            script_path: Arc::new(script_path),
        })
    }

    pub fn to_command(&self) -> Command {
        Command::new(self.script_path.to_path_buf())
    }
}
