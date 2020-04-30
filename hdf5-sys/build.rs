use std::env;
use std::error::Error;
use std::fmt::{self, Debug, Display};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

use regex::Regex;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub micro: u8,
}

impl Version {
    pub fn new(major: u8, minor: u8, micro: u8) -> Self {
        Self { major, minor, micro }
    }

    pub fn parse(s: &str) -> Option<Self> {
        let re = Regex::new(r"^(1)\.(8|10)\.(\d\d?)(_\d+)?(-patch\d+)?$").ok()?;
        let captures = re.captures(s)?;
        Some(Self {
            major: captures.get(1).and_then(|c| c.as_str().parse::<u8>().ok())?,
            minor: captures.get(2).and_then(|c| c.as_str().parse::<u8>().ok())?,
            micro: captures.get(3).and_then(|c| c.as_str().parse::<u8>().ok())?,
        })
    }

    pub fn is_valid(self) -> bool {
        self.major == 1 && ((self.minor == 8 && self.micro >= 4) || (self.minor == 10))
    }
}

impl Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.micro)
    }
}

#[allow(dead_code)]
fn run_command(cmd: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(cmd).args(args).output();
    if let Ok(ref r1) = out {
        if r1.status.success() {
            let r2 = String::from_utf8(r1.stdout.clone());
            if let Ok(r3) = r2 {
                return Some(r3.trim().to_string());
            }
        }
    }
    None
}

#[allow(dead_code)]
fn is_inc_dir<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().join("H5pubconf.h").is_file() || path.as_ref().join("H5pubconf-64.h").is_file()
}

#[allow(dead_code)]
fn is_root_dir<P: AsRef<Path>>(path: P) -> bool {
    is_inc_dir(path.as_ref().join("include"))
}

#[derive(Clone, Debug)]
struct RuntimeError(String);

impl Error for RuntimeError {}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HDF5 runtime error: {}", self.0)
    }
}


#[derive(Clone, Copy, Debug, Default)]
pub struct Header {
    pub have_stdbool_h: bool,
    pub have_direct: bool,
    pub have_parallel: bool,
    pub have_threadsafe: bool,
    pub version: Version,
}

impl Header {
    pub fn parse<P: AsRef<Path>>(inc_dir: P) -> Self {
        let inc_dir = inc_dir.as_ref();

        let header = get_conf_header(inc_dir);
        println!("Parsing HDF5 config from:\n    {:?}", header);

        let contents = fs::read_to_string(header).unwrap();
        let mut hdr = Self::default();

        let num_def_re = Regex::new(r"(?m)^#define\s+(H5_[A-Z_]+)\s+([0-9]+)\s*$").unwrap();
        for captures in num_def_re.captures_iter(&contents) {
            let name = captures.get(1).unwrap().as_str();
            let value = captures.get(2).unwrap().as_str().parse::<i64>().unwrap();
            if name == "H5_HAVE_STDBOOL_H" {
                hdr.have_stdbool_h = value > 0;
            } else if name == "H5_HAVE_DIRECT" {
                hdr.have_direct = value > 0;
            } else if name == "H5_HAVE_PARALLEL" {
                hdr.have_parallel = value > 0;
            } else if name == "H5_HAVE_THREADSAFE" {
                hdr.have_threadsafe = value > 0;
            }
        }

        let str_def_re = Regex::new(r#"(?m)^#define\s+(H5_[A-Z_]+)\s+"([^"]+)"\s*$"#).unwrap();
        for captures in str_def_re.captures_iter(&contents) {
            let name = captures.get(1).unwrap().as_str();
            let value = captures.get(2).unwrap().as_str();
            if name == "H5_VERSION" {
                if let Some(version) = Version::parse(value) {
                    hdr.version = version;
                } else {
                    panic!("Invalid H5_VERSION: {:?}", value);
                }
            }
        }

        if !hdr.version.is_valid() {
            panic!("Invalid H5_VERSION in the header: {:?}", hdr.version);
        }
        hdr
    }
}

fn get_conf_header<P: AsRef<Path>>(inc_dir: P) -> PathBuf {
    let inc_dir = inc_dir.as_ref();

    if inc_dir.join("H5pubconf.h").is_file() {
        inc_dir.join("H5pubconf.h")
    } else if inc_dir.join("H5pubconf-64.h").is_file() {
        inc_dir.join("H5pubconf-64.h")
    } else {
        panic!("H5pubconf header not found in include directory");
    }
}


#[derive(Clone, Debug)]
pub struct Config {
    pub inc_dir: PathBuf,
    pub link_paths: Vec<PathBuf>,
    pub header: Header,
}

impl Config {
    pub fn emit_link_flags(&self) {
        println!("cargo:rustc-link-lib=dylib=hdf5");
        for dir in &self.link_paths {
            println!("cargo:rustc-link-search=native={}", dir.to_str().unwrap());
        }
        println!("cargo:rerun-if-env-changed=HDF5_DIR");
        println!("cargo:rerun-if-env-changed=HDF5_VERSION");
    }

    pub fn emit_cfg_flags(&self) {
        let version = self.header.version;
        assert!(version >= Version::new(1, 8, 4), "required HDF5 version: >=1.8.4");
        let mut vs: Vec<_> = (5..=21).map(|v| Version::new(1, 8, v)).collect(); // 1.8.[5-21]
        vs.extend((0..=5).map(|v| Version::new(1, 10, v))); // 1.10.[0-5]
        for v in vs.into_iter().filter(|&v| version >= v) {
            println!("cargo:rustc-cfg=hdf5_{}_{}_{}", v.major, v.minor, v.micro);
        }
        if self.header.have_stdbool_h {
            println!("cargo:rustc-cfg=h5_have_stdbool_h");
        }
        if self.header.have_direct {
            println!("cargo:rustc-cfg=h5_have_direct");
        }
        if self.header.have_parallel {
            println!("cargo:rustc-cfg=h5_have_parallel");
        }
        if self.header.have_threadsafe {
            println!("cargo:rustc-cfg=h5_have_threadsafe");
        }
    }
}

fn main() {
    if let Ok(hdf5_base_dir_str) = env::var("HDF5_DIR") {
        let hdf5_base_dir = PathBuf::from(hdf5_base_dir_str);
        if !hdf5_base_dir.is_dir() {
            panic!("HDF5_DIR directory: {:?} does not exist", hdf5_base_dir);
        }
        let inc_dir = hdf5_base_dir.join("include");
        if !inc_dir.is_dir() {
            panic!("Include directory: {:?} does not exist", inc_dir);
        }
        let lib_dir = hdf5_base_dir.join("lib");
        if !lib_dir.is_dir() {
            panic!("Static lib directory: {:?} does not exist", lib_dir);
        }
        let config = Config{
            header: Header::parse(&inc_dir),
            inc_dir,
            link_paths: vec![lib_dir],
        };
        println!("CONFIG: \n {:#?}", config);
        config.emit_link_flags();
        config.emit_cfg_flags();
    } else {
        panic!("Environment variable: HDF5_DIR pointing to build directory does not exist");
    }

}
