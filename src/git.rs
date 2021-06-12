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
    let branch = branch.as_str().unwrap_or("refs/heads/master");

    remote.fetch(&[branch], None, None)?;

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;

    let analysis = repo.merge_analysis(&[&fetch_commit])?;

    if analysis.0.is_up_to_date() {
        return Ok(())
    } else if !analysis.0.is_fast_forward() {
        anyhow::bail!("Can't fast forward")
    }

    let mut reference = repo.find_reference(branch)?;
    reference.set_target(fetch_commit.id(), "Fast-Forward")?;
    repo.set_head(branch)?;
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;

    Ok(())
}
