use std::{ fs, io };
use argh::FromArgs;
use crate::profile::Config;


/// Uninstall theme
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "uninstall")]
pub struct Options {
    /// theme repo name
    #[argh(positional)]
    repo: Option<String>,

    /// uninstall all theme
    #[argh(switch)]
    all: bool
}

impl Options {
    pub fn exec(self, config: &Config) -> anyhow::Result<()> {
        if self.repo.is_none() && !self.all {
            anyhow::bail!("nothing");
        }

        let mut repos = config.repos()?;

        if let Some(repo) = self.repo.as_ref() {
            repos.retain(|path| path.ends_with(repo));
        }

        for repo in repos {
            fs::remove_dir_all(&repo)
                .or_else(|err| if err.kind() == io::ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(err)
                })?;

            let name = repo.strip_prefix(config.projdir.data_dir())
                .ok()
                .unwrap_or(&repo);

            println!("{}: remove ok", name.display());
        }

        Ok(())
    }
}
