use argh::FromArgs;
use crate::profile::Config;
use crate::git;


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
            git::pull(&repo)?;

            let name = repo.strip_prefix(config.projdir.data_dir())
                .ok()
                .unwrap_or(&repo);

            println!("{}: update ok", name.display());
        }

        Ok(())
    }
}
