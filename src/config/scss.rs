use std::fs;
use std::path::PathBuf;

pub fn load_css_string() -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let config_dir = PathBuf::from(home).join(".config/riftbar");

    let scss_path = config_dir.join("style.scss");
    let css_path = config_dir.join("style.css");

    // Prefer SCSS if it exists
    if scss_path.exists() {
        let scss = fs::read_to_string(&scss_path).ok()?;

        let options = grass::Options::default().load_path(&config_dir);

        match grass::from_string(scss, &options) {
            Ok(css) => return Some(css),
            Err(err) => {
                eprintln!("SCSS parse error: {err}");
                return None;
            }
        }
    }

    // Fallback to plain CSS
    if css_path.exists() {
        return fs::read_to_string(&css_path).ok();
    }

    // No style file is not an error
    None
}
