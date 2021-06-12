use std::path::Path;


#[cfg(not(feature = "git2"))]
pub fn clone(url: &str, path: &Path) -> anyhow::Result<()> {
    let status = std::process::Command::new("git")
        .arg("clone")
        .arg(url)
        .arg(path)
        .status()?;

    if !status.success() {
        anyhow::bail!("git clone failed: {:?}", status);
    }

    Ok(())
}

#[cfg(feature = "git2")]
pub fn clone(url: &str, path: &Path) -> anyhow::Result<()> {
    git2::Repository::clone(url, path)?;
    Ok(())
}

#[cfg(not(feature = "git2"))]
pub fn pull(path: &Path) -> anyhow::Result<()> {
    let status = std::process::Command::new("git")
        .current_dir(path)
        .arg("pull")
        .arg("-r")
        .status()?;

    if !status.success() {
        anyhow::bail!("git pull failed: {:?}", status);
    }

    Ok(())
}

#[cfg(feature = "git2")]
pub fn pull(path: &Path) -> anyhow::Result<()> {
    let repo = git2::Repository::open(path)?;
    let mut remote = repo.find_remote("origin")?;
    remote.connect(git2::Direction::Fetch)?;
    let branch = remote.default_branch()?;
    let branch = branch.as_str().unwrap_or("master");
    remote.fetch(&[branch], None, None)?;
    repo.set_head(branch)?;
    repo.checkout_head(None)?;

    Ok(())
}
