use rand::{Rng, distr::Alphanumeric, rng};

const BRANCH: &str = "dist-2";

pub fn main() {
    use git2::Repository;
    let mut tmp = std::env::temp_dir().join("arma3-wiki");
    if std::env::var("CI").is_ok() {
        if !tmp.exists() {
            fs_err::create_dir_all(&tmp).expect("Failed to create temp dir");
        }
        let random: String = rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        tmp.push(random);
    }
    let repo = Repository::open(&tmp).unwrap_or_else(|_| {
        git2::build::RepoBuilder::new()
            .branch(BRANCH)
            .clone("https://github.com/acemod/arma3-wiki", &tmp)
            .map_err(|e| format!("Failed to clone repository: {e}"))
            .expect("Failed to clone repository")
    });
    repo.find_remote("origin")
        .and_then(|mut r| r.fetch(&[BRANCH], None, None))
        .map_err(|e| format!("Failed to fetch remote: {e}"))
        .expect("Failed to fetch remote");
    let fetch_head = repo
        .find_reference("FETCH_HEAD")
        .map_err(|e| format!("Failed to find FETCH_HEAD: {e}"))
        .expect("Failed to find FETCH_HEAD");
    let commit = repo
        .reference_to_annotated_commit(&fetch_head)
        .map_err(|e| format!("Failed to find FETCH_HEAD: {e}"))
        .expect("Failed to find FETCH_HEAD");
    let analysis = repo
        .merge_analysis(&[&commit])
        .map_err(|e| format!("Failed to analyze merge: {e}"))
        .expect("Failed to analyze merge");
    if !analysis.0.is_up_to_date() && analysis.0.is_fast_forward() {
        let mut reference = repo
            .find_reference(format!("refs/heads/{BRANCH}").as_str())
            .map_err(|e| format!("Failed to find reference: {e}"))
            .expect("Failed to find reference");
        reference
            .set_target(commit.id(), "Fast-Forward")
            .map_err(|e| format!("Failed to set reference: {e}"))
            .expect("Failed to set reference");
        repo.set_head(format!("refs/heads/{BRANCH}").as_str())
            .map_err(|e| format!("Failed to set HEAD: {e}"))
            .expect("Failed to set HEAD");
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
            .map_err(|e| format!("Failed to checkout HEAD: {e}"))
            .expect("Failed to checkout HEAD");
    }
    let dst = {
        let target_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
        std::path::Path::new(&target_dir).join("arma3-wiki")
    };
    let _ = fs_err::remove_dir_all(&dst);
    fs_extra::dir::copy(
        &tmp,
        dst,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )
    .expect("Failed to copy directory");
    if std::env::var("CI").is_ok() {
        // we sometimes don't have permission? so don't unwrap
        let _ = fs_err::remove_dir_all(tmp);
    }
}
