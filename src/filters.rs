
pub fn rmdashes(title: &str) -> askama::Result<String> {
    Ok(title.replace("-", " ").into())
}
