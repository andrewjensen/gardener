use std::env;
use std::path::PathBuf;

pub fn get_view_path(view_name: &str) -> PathBuf {
    let mut view_path = env::current_dir().unwrap();
    view_path.extend(["public", "views", &format!("{}.html", view_name)].iter());

    view_path
}
