use std::fmt::{self, Display};

use super::{
    debug_already_fetched, info_fetched, info_installed, info_pinned, info_project_version, Tool,
};
use crate::error::ErrorDetails;
use crate::inventory::yarn_available;
use crate::session::Session;
use crate::style::tool_version;
use semver::Version;
use volta_fail::Fallible;

mod fetch;
mod resolve;
mod serial;

pub use resolve::resolve;

/// The Tool implementation for fetching and installing Yarn
pub struct Yarn {
    pub(super) version: Version,
}

impl Yarn {
    pub fn new(version: Version) -> Self {
        Yarn { version }
    }

    pub fn archive_basename(version: &str) -> String {
        format!("yarn-v{}", version)
    }

    pub fn archive_filename(version: &str) -> String {
        format!("{}.tar.gz", Yarn::archive_basename(version))
    }

    pub(crate) fn ensure_fetched(&self, session: &mut Session) -> Fallible<()> {
        if yarn_available(&self.version)? {
            debug_already_fetched(self);
            Ok(())
        } else {
            fetch::fetch(&self.version, session.hooks()?.yarn())
        }
    }
}

impl Tool for Yarn {
    fn fetch(self: Box<Self>, session: &mut Session) -> Fallible<()> {
        self.ensure_fetched(session)?;

        info_fetched(self);
        Ok(())
    }
    fn install(self: Box<Self>, session: &mut Session) -> Fallible<()> {
        self.ensure_fetched(session)?;

        session
            .toolchain_mut()?
            .set_active_yarn(Some(self.version.clone()))?;

        info_installed(self);

        if let Ok(Some(project)) = session.project_platform() {
            if let Some(yarn) = &project.yarn {
                info_project_version(tool_version("yarn", yarn));
            }
        }
        Ok(())
    }
    fn pin(self: Box<Self>, session: &mut Session) -> Fallible<()> {
        if session.project()?.is_some() {
            self.ensure_fetched(session)?;

            // Note: We know this will succeed, since we checked above
            let project = session.project_mut()?.unwrap();
            project.pin_yarn(Some(self.version.clone()))?;

            info_pinned(self);
            Ok(())
        } else {
            Err(ErrorDetails::NotInPackage.into())
        }
    }
}

impl Display for Yarn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&tool_version("yarn", &self.version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yarn_archive_basename() {
        assert_eq!(Yarn::archive_basename("1.2.3"), "yarn-v1.2.3");
    }

    #[test]
    fn test_yarn_archive_filename() {
        assert_eq!(Yarn::archive_filename("1.2.3"), "yarn-v1.2.3.tar.gz");
    }
}
