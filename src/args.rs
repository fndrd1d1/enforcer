use std::cmp;
use std::ops;
use std::num;
use std::process;

use clap;
use app;
use num_cpus;

use std::path::{Path, PathBuf};
use std::result::Result;

// use log;


/// `Args` are transformed/normalized from `ArgMatches`.
#[derive(Debug)]
pub struct Args {
    path: PathBuf,
    endings: Vec<String>,
    clean: bool,
    line_length: Option<usize>,
    color: bool,
    threads: usize,
    quiet: bool,
    status: bool,
    tabs: bool,
}

impl Args {
    /// Parse the command line arguments for this process.
    ///
    /// If a CLI usage error occurred, then exit the process and print a usage
    /// or error message. Similarly, if the user requested the version of
    /// enforcer, then print the version and exit.
    ///
    /// Also, initialize a global logger.
    pub fn parse() -> Result<Args, num::ParseIntError> {
        let matches = app::app().get_matches();
        if matches.is_present("help") {
            let _ = ::app::app().print_help();
            println!("");
            process::exit(0);
        }
        if matches.is_present("version") {
            println!("enforcer {}", crate_version!());
            process::exit(0);
        }
        ArgMatches(matches).to_args()
    }

    /// Whether enforcer should be quiet or not.
    pub fn quiet(&self) -> bool {
        self.quiet
    }
    /// Return the path that should be searched.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn clean(&self) -> bool {
        self.clean
    }
    pub fn status(&self) -> bool {
        self.status
    }
    pub fn tabs(&self) -> bool {
        self.tabs
    }

    /// Returns true if and only if enforcer should color its output.
    pub fn color(&self) -> bool {
        self.color
    }

    /// Return the endings that should be searched.
    pub fn endings(&self) -> &[String] {
        &self.endings
    }

    pub fn threads(&self) -> usize {
        self.threads
    }

    pub fn line_length(&self) -> Option<usize> {
        self.line_length
    }
}

/// `ArgMatches` wraps `clap::ArgMatches` and provides semantic meaning to
/// several options/flags.
struct ArgMatches<'a>(clap::ArgMatches<'a>);

impl<'a> ops::Deref for ArgMatches<'a> {
    type Target = clap::ArgMatches<'a>;
    fn deref(&self) -> &clap::ArgMatches<'a> { &self.0 }
}

impl<'a> ArgMatches<'a> {
    /// Convert the result of parsing CLI arguments into enforcer's
    /// configuration.
    fn to_args(&self) -> Result<Args, num::ParseIntError> {
        let path = self.path();
        let endings = self.endings();
        let quiet = self.is_present("quiet");
        let args = Args {
            path: path,
            endings: endings,
            clean: self.is_present("clean"),
            line_length: try!(self.usize_of("L")),
            color: self.is_present("color"),
            quiet: quiet,
            threads: try!(self.threads()),
            status: self.is_present("status"),
            tabs: self.is_present("tabs"),
        };
        Ok(args)
    }

    /// Return all file endings that enforcer should search.
    fn endings(&self) -> Vec<String> {
        let endings: Vec<String> = match self.values_of_lossy("endings") {
            None => vec![],
            Some(vals) => vals,
        };
        endings
    }

    /// Return all file path that enforcer should search.
    fn path(&self) -> PathBuf {
        match self.value_of_os("path") {
            None => self.default_path(),
            Some(val) => Path::new(val).to_path_buf()
        }
    }

    /// Return the default path that enforcer should search.
    fn default_path(&self) -> PathBuf {
        Path::new("./").to_path_buf()
    }

    /// Returns the approximate number of threads that enforcer should use.
    fn threads(&self) -> Result<usize, num::ParseIntError> {
        let threads = try!(self.usize_of("N")).unwrap_or(0);
        Ok(if threads == 0 {
            cmp::min(12, num_cpus::get())
        } else {
            threads
        })
    }

    /// Safely reads an arg value with the given name, and if it's present,
    /// tries to parse it as a usize value.
    fn usize_of(&self, name: &str) -> Result<Option<usize>, num::ParseIntError> {
        println!("usize_of: {}, {:?}", name, self.0.value_of_lossy(name));
        match self.0.value_of_lossy(name) {
            None => Ok(None),
            Some(v) => v.parse().map(Some).map_err(From::from),
        }
    }
}

