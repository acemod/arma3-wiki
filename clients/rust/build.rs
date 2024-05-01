pub fn main() {
    use git2::Repository;
    let mut tmp = std::env::temp_dir().join("arma3-wiki");
    if std::env::var("CI").is_ok() {
        use rand::distributions::Alphanumeric;
        use rand::{thread_rng, Rng};
        let random: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        tmp.push(random);
    }
    let repo = Repository::open(&tmp).map_or_else(
        |_| {
            git2::build::RepoBuilder::new()
                .branch("dist")
                .clone("https://github.com/acemod/arma3-wiki", &tmp)
                .map_err(|e| format!("Failed to clone repository: {e}"))
                .unwrap()
        },
        |repo| repo,
    );
    repo.find_remote("origin")
        .and_then(|mut r| r.fetch(&["dist"], None, None))
        .map_err(|e| format!("Failed to fetch remote: {e}"))
        .unwrap();
    let fetch_head = repo
        .find_reference("FETCH_HEAD")
        .map_err(|e| format!("Failed to find FETCH_HEAD: {e}"))
        .unwrap();
    let commit = repo
        .reference_to_annotated_commit(&fetch_head)
        .map_err(|e| format!("Failed to find FETCH_HEAD: {e}"))
        .unwrap();
    let analysis = repo
        .merge_analysis(&[&commit])
        .map_err(|e| format!("Failed to analyze merge: {e}"))
        .unwrap();
    if !analysis.0.is_up_to_date() && analysis.0.is_fast_forward() {
        let mut reference = repo
            .find_reference("refs/heads/dist")
            .map_err(|e| format!("Failed to find reference: {e}"))
            .unwrap();
        reference
            .set_target(commit.id(), "Fast-Forward")
            .map_err(|e| format!("Failed to set reference: {e}"))
            .unwrap();
        repo.set_head("refs/heads/dist")
            .map_err(|e| format!("Failed to set HEAD: {e}"))
            .unwrap();
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
            .map_err(|e| format!("Failed to checkout HEAD: {e}"))
            .unwrap();
    }
    let dst = {
        let target_dir = std::env::var("OUT_DIR").unwrap();
        let target_dir = std::path::Path::new(&target_dir).join("arma3-wiki");
        target_dir
    };
    let _ = std::fs::remove_dir_all(&dst);
    fs_extra::dir::copy(
        &tmp,
        dst,
        &fs_extra::dir::CopyOptions::new().content_only(true),
    )
    .unwrap();
    if std::env::var("CI").is_ok() {
        std::fs::remove_dir_all(tmp).unwrap();
    }
}
