#![allow(unused)]

use serde_json::Value;
use std::{
    fmt, io,
    path::PathBuf,
    process::{Command, Stdio},
};

/// Returns true if able to spawn a vim process
pub fn has_vim_on_path() -> bool {
    has_on_path("vim")
}

/// Returns true if able to spawn an nvim process
pub fn has_nvim_on_path() -> bool {
    has_on_path("nvim")
}

fn has_on_path(cmd: &str) -> bool {
    !matches!(
        Command::new(cmd)
            .arg("--help")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn(),
        Err(x) if x.kind() == io::ErrorKind::NotFound
    )
}

/// Performs search to find vimrc based on platform
pub fn find_vimrc() -> Option<PathBuf> {
    if cfg!(unix) {
        let home = shellexpand::tilde("~");
        let path1: PathBuf = [home.as_ref(), ".vimrc"].iter().collect();
        let path2: PathBuf = [home.as_ref(), ".vim", "vimrc"].iter().collect();

        match (path1, path2) {
            (path, _) if path.exists() => Some(path),
            (_, path) if path.exists() => Some(path),
            _ => None,
        }
    } else if cfg!(windows) {
        let home = shellexpand::tilde("~");
        let vim_env = shellexpand::env("$VIM");

        let path1: PathBuf = [home.as_ref(), "_vimrc"].iter().collect();
        let path2: PathBuf =
            [home.as_ref(), "vimfiles", "vimrc"].iter().collect();
        let path3: Option<PathBuf> = vim_env
            .ok()
            .map(|vim| [vim.as_ref(), "_vimrc"].iter().collect());

        match (path1, path2, path3) {
            (path, _, _) if path.exists() => Some(path),
            (_, path, _) if path.exists() => Some(path),
            (_, _, Some(path)) if path.exists() => Some(path),
            _ => None,
        }
    } else {
        None
    }
}

/// Represents a vim variable to be extracted
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VimVar<VarName: AsRef<str>> {
    cmd: VimCmd,
    scope: VimVarScope,
    var_name: VarName,
}

impl<VarName: AsRef<str>> VimVar<VarName> {
    pub fn new(cmd: VimCmd, scope: VimVarScope, var_name: VarName) -> Self {
        Self {
            cmd,
            scope,
            var_name,
        }
    }
}

impl<VarName: AsRef<str>> VimVar<VarName> {
    /// Retrieves a vim variable with `g:` scope
    pub fn get_global(
        var_name: VarName,
        allow_zero: bool,
    ) -> io::Result<Option<Value>> {
        let cmd = if has_nvim_on_path() {
            VimCmd::Neovim
        } else if has_vim_on_path() {
            VimCmd::Vim
        } else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No vim or neovim instance found in path",
            ));
        };

        Self {
            cmd,
            scope: VimVarScope::Global,
            var_name,
        }
        .execute(allow_zero)
    }

    /// Retrieves a vim variable with `v:` scope
    pub fn get_vim_predefined(
        var_name: VarName,
        allow_zero: bool,
    ) -> io::Result<Option<Value>> {
        let cmd = if has_nvim_on_path() {
            VimCmd::Neovim
        } else if has_vim_on_path() {
            VimCmd::Vim
        } else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No vim or neovim instance found in path",
            ));
        };

        Self {
            cmd,
            scope: VimVarScope::Vim,
            var_name,
        }
        .execute(allow_zero)
    }
}

impl<VarName: AsRef<str>> VimVar<VarName> {
    /// Spawns a vim process whose goal is to print out the contents of a variable
    /// as a JSON string
    ///
    /// Relies on the variable being available upon loading vim configs
    ///
    /// If allow_zero is true, then a value of 0 is considered the value of
    /// the variable rather than vim's default of not being found
    pub fn execute(self, allow_zero: bool) -> io::Result<Option<Value>> {
        let cmd = self.cmd;
        let scope = self.scope.as_vim_str();
        let var = self.var_name.as_ref();

        let full_cmd = match cmd {
            VimCmd::Neovim => format!(
                r#"{} --headless '+echon json_encode(get({}, "{}"))' '+qa!'"#,
                cmd, scope, var,
            ),
            VimCmd::Vim => {
                // NOTE: If vim was started with |-es| all initializations
                //       described in :help initialization are skipped
                //
                //       So, we need to manually re-introduce the vimrc for
                //       vim to load variables that would be relevant
                let vimrc = find_vimrc().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "vimrc not found")
                })?;
                format!(
                    r#"{} -Es -u "{}" '+redir => m | echon json_encode(get({}, "{}")) | redir END | put=m' '+%p' '+qa!'"#,
                    cmd,
                    vimrc.to_string_lossy(),
                    scope,
                    var,
                )
            }
        };

        // TODO: Support windows here (won't have sh)
        let output = Command::new("sh").arg("-c").arg(full_cmd).output()?;

        // NOTE: If using neovim's --headless option, the output appears on
        //       stderr whereas using the redir approach places output on stdout
        let output_string = match self.cmd {
            VimCmd::Vim => String::from_utf8_lossy(&output.stdout),
            VimCmd::Neovim => String::from_utf8_lossy(&output.stderr),
        };

        let value: Value = serde_json::from_str(output_string.trim())
            .map_err(io::Error::from)?;

        if !allow_zero && value == serde_json::json!(0) {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    }
}

/// Represents type of vim instance being used
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum VimCmd {
    Vim,
    Neovim,
}

impl VimCmd {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Vim => "vim",
            Self::Neovim => "nvim",
        }
    }
}

impl fmt::Display for VimCmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Represents a vim variable scope
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum VimVarScope {
    /// `g:`
    Global,
    /// `v:`
    Vim,
}

impl Default for VimVarScope {
    /// Returns global as default
    fn default() -> Self {
        Self::Global
    }
}

impl VimVarScope {
    pub fn as_vim_str(&self) -> &'static str {
        match self {
            Self::Global => "g:",
            Self::Vim => "v:",
        }
    }
}

impl fmt::Display for VimVarScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_vim_str())
    }
}
