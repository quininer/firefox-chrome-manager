use std::{ fs, io };
use std::process::Command;
use argh::FromArgs;
use crate::profile::Config;


/// Install theme
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "install")]
pub struct Options {
    /// theme repo name `username/repo` or url
    #[argh(positional)]
    repo: String,

    /// profile name. if not specified, it will be installed to all default profiles
    #[argh(option)]
    profile: Option<String>,
}

impl Options {
    pub fn exec(self, config: &Config) -> anyhow::Result<()> {
        let mut profiles = config.profiles()?;

        if let Some(name) = self.profile.as_ref() {
            profiles.retain(|profile| &profile.name == name);
        }

        for profile in profiles.iter() {
            profile.check_and_make_prefs(config)?;
        }

        let repo_path = config.projdir.data_dir().join(&self.repo);
        if !repo_path.exists() {
            let status = Command::new("git")
                .current_dir(config.projdir.data_dir())
                .arg("clone")
                .arg(format!("https://github.com/{}", self.repo))
                .arg(&self.repo)
                .status()?;

            if !status.success() {
                anyhow::bail!("git clone failed: {:?}", status);
            }
        }

        for profile in profiles.iter() {
            let chrome_path = config.chrome_path(&profile);

            #[cfg(unix)] {
                use std::os::unix::fs::symlink;

                fs::remove_file(&chrome_path)
                    .or_else(|err| if err.kind() == io::ErrorKind::NotFound {
                        Ok(())
                    } else {
                        Err(err)
                    })?;

                symlink(&repo_path, &chrome_path)?;
            }

            #[cfg(windows)] {
                use std::os::windows::fs::symlink_dir;

                fs::remove_dir_all(&chrome_path)
                    .or_else(|err| if err.kind() == io::ErrorKind::NotFound {
                        Ok(())
                    } else {
                        Err(err)
                    })?;

                // TODO it work?

                symlink_dir(&repo_path, &chrome_path)?;
            }
        }

        Ok(())
    }
}
