use std::process::Command;
use argh::FromArgs;
use crate::profile::Config;


/// Update theme
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "update")]
pub struct Options {
    /// theme repo name `username/repo`
    #[argh(option)]
    repo: Option<String>
}

impl Options {
    pub fn exec(self, config: &Config) -> anyhow::Result<()> {
        let mut repos = config.repos()?;

        if let Some(repo) = self.repo.as_ref() {
            repos.retain(|path| path.ends_with(repo));
        }

        for repo in repos {
            let status = Command::new("git")
                .current_dir(&repo)
                .arg("pull")
                .arg("-r")
                .status()?;

            if !status.success() {
                anyhow::bail!("git pull failed: {:?}", status);
            }
        }

        Ok(())
    }
}
