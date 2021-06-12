use std::{ fs, io };
use argh::FromArgs;
use url::Url;
use crate::profile::Config;
use crate::git;


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

        let mut repo_path = Box::new(config.projdir.data_dir()).to_path_buf();

        let repo_url = match Url::parse(&self.repo) {
            Ok(url) => {
                if let Some(domain) = url.domain()
                    .filter(|&domain| domain != "github.com")
                {
                    repo_path.push(domain);
                }
                repo_path.push(url.path().trim_matches('/'));
                self.repo
            },
            Err(_) => {
                repo_path.push(&self.repo);
                format!("https://github.com/{}", self.repo)
            }
        };

        let repo_name = repo_path.strip_prefix(config.projdir.data_dir())
            .ok()
            .unwrap_or(&repo_path);

        if !repo_path.exists() {
            if let Some(path) = repo_path.parent() {
                fs::create_dir_all(path)
                    .or_else(|err| if err.kind() == io::ErrorKind::AlreadyExists {
                        Ok(())
                    } else {
                        Err(err)
                    })?;
            }

            git::clone(&repo_url, &repo_path)?;

            println!("{}: clone ok", repo_name.display());
        }

        for profile in profiles.iter() {
            let chrome_path = config.chrome_path(profile);

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

            println!("install {} for {}", repo_name.display(), profile.name);
        }

        Ok(())
    }
}
