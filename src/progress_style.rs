use indicatif::ProgressStyle;

pub struct ProgressStyles {}

impl ProgressStyles {
    pub fn progress_style() -> ProgressStyle {
        ProgressStyle::default_bar()
            .template("{prefix:.bold.dim} {msg} [{bar:70.cyan/blue}] {pos}/{len} ")
            .progress_chars("#>-")
    }

    pub fn spinner_style() -> ProgressStyle {
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{prefix:.bold.dim} {msg} {spinner} ")
    }
}
