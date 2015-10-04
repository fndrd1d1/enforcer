extern crate enforcer;
extern crate rustc_serialize;
extern crate docopt;
extern crate glob;
#[macro_use] extern crate log;
extern crate env_logger;

use enforcer::check;
use std::fs::File;
use std::io::Read;
use docopt::Docopt;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const USAGE: &'static str = "
enforcer for code rules

Usage:
  enforcer [-g GLOB...] [-c|--clean]
  enforcer (-h | --help)
  enforcer (-v | --version)

Options:
  -g GLOB       use these glob patterns (e.g. \"**/*.h\")
  -h --help     Show this screen.
  -v --version  Show version.
  --count       only count found entries
  -c --clean    clean up trailing whitespaces
";
#[derive(Debug, RustcDecodable)]
struct Args {
    flag_count: bool,
    flag_clean: bool,
    flag_g: Vec<String>,
    flag_version: bool,
}

#[allow(dead_code)]
fn main() {
    use glob::glob;
    env_logger::init().unwrap();
    debug!("check content of");

    fn get_cfg() -> check::EnforcerCfg {
        fn read_enforcer_config() -> std::io::Result<check::EnforcerCfg> {
            let mut cfg_file = try!(File::open(".enforcer"));
            let mut buffer = String::new();
            try!(cfg_file.read_to_string(&mut buffer));
            check::read_config(&buffer[..])
        }
        let enforcer_cfg = read_enforcer_config()
            .unwrap_or(check::default_cfg());
        enforcer_cfg
    }

    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("  Version: {}", VERSION);
    }
    let enforcer_cfg = get_cfg();
    let cfg_ignores = enforcer_cfg.ignore;
    let cfg_globs = enforcer_cfg.globs;
    let pats = if args.flag_g.len() > 0 {
        args.flag_g
    } else {
        cfg_globs
    };

    fn find_matches<'a>(pat: &str, to_ignore: &Vec<String>) -> Vec<std::path::PathBuf> {
        glob(&*pat) // -> Result<Paths, PatternError>
            .ok()   // -> Option<Paths>
            .expect(&format!("glob has problems with {}", pat)[..]) // -> Paths (Iterator ofer GlobResult)
            .filter_map(Result::ok) // ignore unreadable paths -> Iterator over PathBuf
            .filter(|x| !x.components()
                        .any(|y| check::is_unwanted(y, to_ignore))).collect()
    }
    let paths: Vec<std::path::PathBuf> = pats.iter().flat_map(|pat| find_matches(pat, &cfg_ignores)).collect();
    let mut checked_files: u32 = 0;
    let mut had_tabs: u32 = 0;
    let mut had_illegals: u32 = 0;
    for path in paths {
        if !check::is_dir(path.as_path()) {
            checked_files += 1;
            let r = check::check_path(path.as_path(), args.flag_clean)
                .ok()
                .expect(&format!("check_path for {:?} should work", path));
            if (r & check::HAS_TABS) > 0 { had_tabs += 1 }
            if (r & check::HAS_ILLEGAL_CHARACTERS) > 0 { had_illegals += 1 }
        }
    }
    println!("checked {} files! ({} had tabs, {} had illegal characters)", checked_files, had_tabs, had_illegals);
}

#[cfg(test)]
mod tests {
    extern crate enforcer;
    use enforcer::check::clean_string;
    use enforcer::check::read_config;
    use enforcer::check::is_unwanted;
    use enforcer::check::s;
    use glob::Pattern;
    use enforcer::check::EnforcerCfg;
    use std::ffi::OsStr;
    use std::path::Component::Normal;

    #[test]
    fn test_clean_does_not_remove_trailing_newline() {
        let content = "1\n2\n3\n4\n5\n";
        let cleaned = clean_string(content);
        assert!(cleaned.eq(content));
    }
    #[test]
    fn test_clean_trailing_whitespace() {
        let content = "1 \n2";
        let cleaned = clean_string(content);
        println!("{:?}", cleaned);
        assert!(cleaned.eq("1\n2"));
    }
    #[test]
    fn test_load_simple_config() {
        let c = include_str!("../samples/.enforcer");
        let cfg = read_config(c).unwrap();
        assert_eq!(cfg.ignore.len(), 2);
        let expected = EnforcerCfg {
            ignore: vec![s(".git"), s(".repo")],
            globs : vec![s("**/*.c"), s("**/*.cpp"), s("**/*.h")],
        };
        assert_eq!(expected.ignore, cfg.ignore);
        assert_eq!(expected, cfg);
    }
    #[test]
    fn test_load_broken_config() {
        let c = include_str!("../samples/.enforcer_broken");
        let cfg = read_config(c).unwrap();
        let expected = EnforcerCfg {
            ignore: vec![s(".git"), s(".repo")],
            globs : vec![s("**/*.c"), s("**/*.cpp"), s("**/*.h")],
        };
        assert!(expected.ignore != cfg.ignore);
    }
    #[test]
    fn test_glob() {
        assert!(Pattern::new("build_*").unwrap().matches("build_Debug"));
    }
    #[test]
    fn test_is_unwanted() {
        let cfg = EnforcerCfg { ignore: vec![s("build_*"), s(".git")], globs: vec![]};
        assert!(is_unwanted(Normal(OsStr::new("build_Debug")), &cfg.ignore));
        assert!(is_unwanted(Normal(OsStr::new(".git")), &cfg.ignore));
        assert!(!is_unwanted(Normal(OsStr::new("bla")), &cfg.ignore));
    }
}

