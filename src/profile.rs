use std::{ fs, io };
use std::fmt::Write;
use std::path::PathBuf;
use anyhow::Context;
use directories::{ UserDirs, ProjectDirs };
use ini::Ini;

pub struct Config {
    pub userdir: UserDirs,
    pub projdir: ProjectDirs
}

#[derive(Debug)]
pub struct Profile {
    pub name: String,
    pub path: String
}

impl Config {
    pub fn new() -> anyhow::Result<Config> {
        Ok(Config {
            userdir: UserDirs::new()
                .context("Unable to retrieve user home from system")?,
            projdir: ProjectDirs::from("", "", env!("CARGO_PKG_NAME"))
                .context("Unable to retrieve project path from system")?
        })
    }

    pub fn chrome_path(&self, profile: &Profile) -> PathBuf {
        self.userdir.home_dir()
            .join(".mozilla")
            .join("firefox")
            .join(&profile.path)
            .join("chrome")
    }

    pub fn repos(&self) -> anyhow::Result<Vec<PathBuf>> {
        let mut output = Vec::new();

        for entry in walkdir::WalkDir::new(self.projdir.data_dir())
            .min_depth(2)
            .max_depth(2)
            .follow_links(true)
        {
            let entry = entry?;

            if !entry.file_type().is_dir() {
                continue;
            }

            if !entry.path().join(".git").exists() {
                continue;
            }

            output.push(entry.into_path());
        }

        Ok(output)
    }

    pub fn profiles(&self) -> anyhow::Result<Vec<Profile>> {
        let profiles_path = self.userdir.home_dir()
            .join(".mozilla")
            .join("firefox")
            .join("profiles.ini");

        let mut profiles = Vec::new();

        // Should escape be turned off for windows?
        let ini = Ini::load_from_file(&profiles_path)?;

        for (name, prop) in ini.iter() {
            if name.filter(|name| name.starts_with("Profile"))
                .is_none()
            {
                continue;
            }

            if let (Some(name), Some(path)) = (prop.get("Name"), prop.get("Path")) {
                profiles.push(Profile {
                    name: name.to_owned(),
                    path: path.to_owned()
                });
            }
        }

        if profiles.is_empty() {
            anyhow::bail!("No found any profile");
        }

        Ok(profiles)
    }
}

impl Profile {
    pub fn check_and_make_prefs(&self, config: &Config) -> anyhow::Result<()> {
        macro_rules! try_continue {
            ( $item:expr ) => {
                if let Some(item) = $item {
                    item
                } else {
                    continue
                }
            }
        }

        let prefs_path = config.userdir.home_dir()
            .join(".mozilla")
            .join("firefox")
            .join(&self.path)
            .join("prefs.js");

        let buf = match fs::read_to_string(&prefs_path) {
            Ok(buf) => buf,
            Err(ref err) if err.kind() == io::ErrorKind::NotFound => String::new(),
            Err(err) => return Err(err.into())
        };
        let mut checklist = [
            ("toolkit.legacyUserProfileCustomizations.stylesheets", false),
            ("layers.acceleration.force-enabled", false),
            ("gfx.webrender.all", false),
            ("gfx.webrender.enabled", false),
            ("layout.css.backdrop-filter.enabled", false),
            ("svg.context-properties.content.enabled", false)
        ];

        // check
        for line in buf.lines() {
            let line = line.trim();
            if line.starts_with("//") || line.is_empty() {
                continue;
            }

            let line = try_continue!(line.strip_prefix("user_pref"));
            let line = try_continue!(line.strip_suffix(";"));
            let line = line.trim();
            let line = try_continue!(line.strip_prefix("("));
            let line = try_continue!(line.strip_suffix(")"));
            let line = line.trim();
            let (name, val) = try_continue!(line.split_once(','));
            let name = name.trim();
            let val = val.trim();
            let name = try_continue!(name.strip_prefix('"'));
            let name = try_continue!(name.strip_suffix('"'));

            if val.parse::<bool>() != Ok(true) {
                continue
            }

            for (check_name, val) in checklist.iter_mut() {
                if *check_name == name {
                    *val = true;
                }
            }
        }

        if checklist.iter().all(|(_, val)| *val) {
            return Ok(());
        }

        // make
        let prefs_bak_path =  config.userdir.home_dir()
            .join(".mozilla")
            .join("firefox")
            .join(&self.path)
            .join("prefs-1.js");
        fs::write(&prefs_bak_path, &buf)?;

        let mut newbuf = buf;
        for (name, val) in checklist.iter() {
            if !val {
                writeln!(&mut newbuf, "user_pref(\"{}\", {})", name, val)?;
            }
        }
        fs::write(&prefs_path, &newbuf)?;

        Ok(())
    }
}
