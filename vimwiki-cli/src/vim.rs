use serde_json::Value;
use std::{
    io,
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

/// Represents a vim variable to be extracted
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VimVar<Cmd: AsRef<str>, VarName: AsRef<str>> {
    cmd: Cmd,
    scope: VimVarScope,
    var_name: VarName,
}

impl<Cmd: AsRef<str>, VarName: AsRef<str>> VimVar<Cmd, VarName> {
    pub fn new(cmd: Cmd, scope: VimVarScope, var_name: VarName) -> Self {
        Self {
            cmd,
            scope,
            var_name,
        }
    }
}

impl<VarName: AsRef<str>> VimVar<&'static str, VarName> {
    /// Retrieves a vim variable with `g:` scope
    pub fn get_global(var_name: VarName) -> io::Result<Value> {
        if !has_nvim_on_path() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Only neovim is supported right now!",
            ));
        }

        Self {
            cmd: "nvim",
            scope: VimVarScope::Global,
            var_name,
        }
        .execute()
    }

    /// Retrieves a vim variable with `v:` scope
    pub fn get_vim_predefined(var_name: VarName) -> io::Result<Value> {
        if !has_nvim_on_path() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Only neovim is supported right now!",
            ));
        }

        Self {
            cmd: "nvim",
            scope: VimVarScope::Vim,
            var_name,
        }
        .execute()
    }
}

impl<Cmd: AsRef<str>, VarName: AsRef<str>> VimVar<Cmd, VarName> {
    /// Spawns a vim process whose goal is to print out the contents of a variable
    /// as a JSON string
    ///
    /// Relies on the variable being available upon loading vim configs
    pub fn execute(self) -> io::Result<Value> {
        let cmd = self.cmd.as_ref();
        let scope = self.scope.as_vim_str();
        let var = self.var_name.as_ref();

        // TODO: Support windows here (won't have /dev/stdout)
        // TODO: Support vim here (doesn't have --headless and using any
        //       other method like +'redir>>/dev/stdout' includes bunch of
        //       control characters)
        let full_cmd = format!(
            r#"{} --headless +'echon json_encode(get({}, "{}", ""))' +qa"#,
            cmd, scope, var,
        );

        // TODO: Support windows here (won't have sh)
        let output = Command::new("sh").arg("-c").arg(full_cmd).output()?;

        serde_json::from_str(String::from_utf8_lossy(&output.stderr).trim())
            .map_err(Into::into)
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
