use osstrtools::OsStrTools;
#[cfg(feature = "log")]
use std::io::Write;
use std::{
    env,
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::Command,
};

pub enum Opt {
    Fast,
    Safe,
    Small,
}

impl Into<String> for &Opt {
    fn into(self) -> String {
        match self {
            Opt::Fast => "ReleaseFast".to_string(),
            Opt::Safe => "ReleaseSafe".to_string(),
            Opt::Small => "ReleaseSmall".to_string(),
        }
    }
}

enum LibType {
    Static,
    Dynamic,
}

pub struct Build {
    file: Option<PathBuf>,
    lib_name: Option<PathBuf>,
    flags: Vec<String>,
    out_dir: Option<PathBuf>,
    out_type: LibType,
    optimiziation: Option<Opt>,

    #[cfg(feature = "log")]
    log_file: Option<std::fs::File>,
}

impl Build {
    pub fn new() -> Self {
        Build {
            file: None,
            lib_name: None,
            flags: vec![],
            out_dir: None,
            out_type: LibType::Dynamic,
            optimiziation: None,

            #[cfg(feature = "log")]
            log_file: None,
        }
    }

    #[cfg(feature = "log")]
    pub fn log(mut self, log: bool) -> Build {
        if log {
            self.log_file = std::fs::OpenOptions::new()
                .append(true)
                .write(true)
                .create(true)
                .open(self.get_log_dir())
                .ok()
        }

        if let Some(log_file) = self.log_file.as_mut() {
            writeln!(log_file).expect("Log file to have been created!");
            writeln!(log_file, "--------------------------------------")
                .expect("Log file to have been created!");
            writeln!(log_file, "T:{}", chrono::Local::now())
                .expect("Log file to have been created!");
            writeln!(log_file, "--------------------------------------")
                .expect("Log file to have been created!");
            writeln!(log_file).expect("Log file to have been created!");
        };

        self
    }

    #[cfg(feature = "log")]
    fn get_log_dir(&self) -> PathBuf {
        self.get_out_dir().join("logs.txt")
    }

    fn get_out_dir(&self) -> PathBuf {
        match self.out_dir.clone() {
            None => env::var_os("OUT_DIR")
                .map(PathBuf::from)
                .expect("OUT_DIR to be set by cargo!"),
            Some(p) => p,
        }
    }

    fn match_target(&mut self) -> String {
        let split_target = std::env::var("TARGET")
            .expect("TARGET to be set by cargo")
            .split('-')
            .map(String::from)
            .collect::<Vec<_>>();

        format!(
            "{}-{}-{}",
            split_target[0], split_target[2], split_target[3]
        )
    }

    pub fn file<P: AsRef<Path>>(mut self, p: P) -> Build {
        self.file = Some(PathBuf::from(p.as_ref()));
        self
    }

    pub fn lib_name<P: AsRef<Path>>(mut self, p: P) -> Build {
        self.lib_name = Some(PathBuf::from(p.as_ref()));
        self
    }

    pub fn flags<F>(mut self, f: F) -> Build
    where
        F: IntoIterator,
        F::Item: AsRef<str>,
    {
        self.flags
            .extend(f.into_iter().map(|s| s.as_ref().to_owned()));
        self
    }

    pub fn as_static(mut self) -> Build {
        self.out_type = LibType::Static;
        self
    }

    pub fn as_dynlib(mut self) -> Build {
        self.out_type = LibType::Dynamic;
        self
    }

    /// Configure the optimization level of the library.
    /// If crate is compiled with `debug` this defaults to `DEBUG`,
    /// otherwise it defaults to `ReleaseSafe`
    pub fn optimiziation(mut self, opt: Opt) -> Build {
        self.optimiziation = Some(opt);
        self
    }

    fn get_lib_name(&self) -> PathBuf {
        match self.lib_name.as_ref() {
            Some(lib_name) => lib_name.clone(),

            None => PathBuf::from(
                self.file
                    .clone()
                    .expect("file to be set!")
                    .file_name()
                    .expect("file to be set")
                    .to_os_string()
                    .split(".")[0],
            ),
        }
    }

    fn get_emit_path(&self) -> PathBuf {
        let lib_name = self.get_lib_name();

        let out_dir = self.get_out_dir();

        out_dir.join(format!("lib{}", lib_name.display()))
    }

    fn set_cargo_search_dir(&mut self) {
        let cmd = format!(
            "cargo:rustc-link-search=native={}",
            self.get_out_dir().display()
        );

        #[cfg(feature = "log")]
        if let Some(log) = self.log_file.as_mut() {
            writeln!(log, "{}", cmd).expect("log file to exist");
        }

        println!("{}", cmd);
    }

    fn get_cargo_out_type(&self) -> String {
        match &self.out_type {
            LibType::Static => "static".to_owned(),
            LibType::Dynamic => "dynlib".to_owned(),
        }
    }

    fn get_zig_out_type(&self) -> String {
        match &self.out_type {
            LibType::Static => "static".to_owned(),
            LibType::Dynamic => "dynamic".to_owned(),
        }
    }

    fn set_cargo_lib_name(&mut self) {
        let cmd = format!(
            "cargo:rustc-link-lib={}={}",
            self.get_cargo_out_type(),
            self.get_lib_name().display()
        );

        #[cfg(feature = "log")]
        if let Some(log) = self.log_file.as_mut() {
            writeln!(log, "{}", cmd).expect("log file to exist");
        }

        println!("{}", cmd);
    }

    fn set_rerun_pref(&self) {
        println!(
            "cargo:rustc-rerun-if-changed={}",
            self.file.as_ref().unwrap().display()
        )
    }

    fn get_lib_ft(&self) -> String {
        match self.out_type {
            LibType::Static => String::from("a"),
            LibType::Dynamic => String::from("so"),
        }
    }

    fn match_profile(&self) -> String {
        if let Some(opt) = self.optimiziation.as_ref() {
            return opt.into();
        }

        match std::env::var("PROFILE")
            .expect("PROFILE to be set by cargo")
            .as_str()
        {
            "release" => "ReleaseSafe".to_string(),
            "debug" => "Debug".to_string(),
            _ => unreachable!("Invalid cargo PROFILE env"),
        }
    }

    pub fn finish(mut self) {
        let Some(file) = self.file.clone() else {
            return;
        };

        if !file.is_file() {
            panic!("Expected file to link to!")
        }

        self.set_cargo_search_dir();

        self.set_cargo_lib_name();

        self.set_rerun_pref();

        Command::new("zig")
            // zig build command
            .arg("build-lib")
            // set to static or dynamic lib
            .arg(format!("-{}", self.get_zig_out_type()))
            // sets the emit path to the OUT_DIR
            .arg(format!(
                "-femit-bin={}.{}",
                self.get_emit_path().display(),
                self.get_lib_ft()
            ))
            // changes the default SONAME to the specified lib name
            .arg(format!(
                "-fsoname=lib{}.{}",
                self.get_lib_name().display(),
                self.get_lib_ft()
            ))
            // set the output directory of the build cache
            .arg("--cache-dir")
            .arg(self.get_out_dir())
            // this is not checking if we are passing a valid target!
            // that we leave to `zig`
            .arg("-target")
            .arg(self.match_target())
            // set the compilation mode
            .arg("-O")
            .arg(self.match_profile())
            // set the file to be compiled and linked
            .arg(self.file.as_ref().expect("lib file to be set"))
            // execute the command
            .exec();
    }
}

#[cfg(test)]
mod tests {
    use osstrtools::OsStrTools;
    use std::fmt::format;

    use super::*;

    #[test]
    fn it_works() {
        let lib_path = PathBuf::from("./src/main.zig");
        println!(
            "buffer: {}",
            PathBuf::from("./deps/")
                .join(lib_path.file_name().unwrap().split(".")[0])
                .display()
        );
    }
}
