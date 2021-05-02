use derive_more::Constructor;
use std::{ffi::OsStr, io, process::Command};

/// Represents an equality check that is considered strict. In the case of
/// a `Located<T>`, will check both the inner type AND the region.
pub trait StrictEq<Rhs: ?Sized = Self> {
    fn strict_eq(&self, other: &Rhs) -> bool;

    #[inline]
    fn strict_ne(&self, other: &Rhs) -> bool {
        !self.strict_eq(other)
    }
}

/// Blanket implementation for two vectors of similarly-typed StrictEq elements
impl<T: StrictEq> StrictEq for Vec<T> {
    /// Performs strict_eq check on inner elements
    fn strict_eq(&self, other: &Self) -> bool {
        self.len() == other.len()
            && self.iter().zip(other.iter()).all(|(x, y)| x.strict_eq(y))
    }
}

/// Represents a vim variable to be extracted
#[derive(Constructor, Clone, Debug, PartialEq, Eq)]
pub struct VimVar<Cmd: AsRef<OsStr>, VarName: AsRef<str>> {
    cmd: Cmd,
    scope: VimVarScope,
    var_name: VarName,
}

impl<VarName: AsRef<str>> VimVar<&'static str, VarName> {
    /// Retrieves a vim variable with `g:` scope
    pub fn get_global(var_name: VarName) -> io::Result<String> {
        Self {
            cmd: "vim",
            scope: VimVarScope::Global,
            var_name,
        }
        .execute()
    }

    /// Retrieves a vim variable with `v:` scope
    pub fn get_vim_predefined(var_name: VarName) -> io::Result<String> {
        Self {
            cmd: "vim",
            scope: VimVarScope::Vim,
            var_name,
        }
        .execute()
    }
}

impl<Cmd: AsRef<OsStr>, VarName: AsRef<str>> VimVar<Cmd, VarName> {
    /// Spawns a vim process whose goal is to print out the contents of a variable
    ///
    /// Relies on the variable being available upon loading vim configs
    pub fn execute(self) -> io::Result<String> {
        let output = Command::new(self.cmd)
            .arg("+'redir>>/dev/stdout'")
            .arg(format!(
                r#"+'echo get({}, "{}", "")'"#,
                self.scope.as_vim_str(),
                self.var_name.as_ref()
            ))
            .arg("+qa")
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
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
