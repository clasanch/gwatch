use clap::Parser;

/// Real-time Git-powered directory monitor with line-by-line diff visualization
#[derive(Parser, Debug)]
#[command(name = "gwatch")]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Directory to watch (defaults to current directory)
    #[arg(short, long, default_value = ".")]
    pub path: String,

    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

impl Args {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_args() {
        let args = Args::parse_from(["gwatch"]);
        assert_eq!(args.path, ".");
        assert_eq!(args.verbose, 0);
    }

    #[test]
    fn test_custom_path() {
        let args = Args::parse_from(["gwatch", "--path", "/tmp/myrepo"]);
        assert_eq!(args.path, "/tmp/myrepo");
    }

    #[test]
    fn test_short_path() {
        let args = Args::parse_from(["gwatch", "-p", "/home/user/project"]);
        assert_eq!(args.path, "/home/user/project");
    }

    #[test]
    fn test_verbose_flags() {
        let args = Args::parse_from(["gwatch", "-v"]);
        assert_eq!(args.verbose, 1);

        let args = Args::parse_from(["gwatch", "-vv"]);
        assert_eq!(args.verbose, 2);

        let args = Args::parse_from(["gwatch", "-vvv"]);
        assert_eq!(args.verbose, 3);
    }

    #[test]
    fn test_combined_args() {
        let args = Args::parse_from(["gwatch", "-p", "/tmp", "-vv"]);
        assert_eq!(args.path, "/tmp");
        assert_eq!(args.verbose, 2);
    }
}
