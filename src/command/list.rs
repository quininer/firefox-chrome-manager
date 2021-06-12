use argh::FromArgs;
use crate::profile::Config;


/// List installed theme or profile
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "list")]
pub struct Options {
    /// list all profile
    #[argh(switch)]
    profile: bool,
}

impl Options {
    pub fn exec(self, config: &Config) -> anyhow::Result<()> {
        if self.profile {
            println!("{:#?}", config.profiles()?);
        } else {
            for repo in config.repos()? {
                let name = repo
                    .strip_prefix(config.projdir.data_dir())?
                    .display();
                println!("{}", name);
            }
        }

        Ok(())
    }
}
