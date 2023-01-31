use osstrtools::OsStrTools;
use std::{
    env,
    io::Write,
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::Command,
};

pub struct Build {
    file: Option<PathBuf>,
    lib_name: Option<PathBuf>,
    flags: Vec<String>,
    out_dir: Option<PathBuf>,
    out_type: LibType,
}

impl Build {
    pub fn new() -> Self {
        Build {
            file: None,
            lib_name: None,
            flags: vec![],
            out_dir: None,
            out_type: LibType::Dynamic,
        }
    }

    fn get_out_dir(&self) -> Result<PathBuf, ()> {
        match self.out_dir.clone() {
            None => Ok(env::var_os("OUT_DIR")
                .map(PathBuf::from)
                .ok_or_else(|| ())?),
            Some(p) => Ok(p),
        }
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

        let out_dir = self.get_out_dir().expect("out dir to be set");

        out_dir.join(format!("lib{}", lib_name.display()))
    }

    fn set_cargo_search_dir(&self) {
        
        std::fs::OpenOptions::new()
            .append(true)
            .write(true)
            .open("./log.txt")
            .unwrap()
            .write(
            format!(
                "cargo:rustc-link-search=native={}\n",
                self.get_out_dir().as_ref().unwrap().display()
            )
            .as_bytes(),
        );
        println!(
            "cargo:rustc-link-search=native={}",
            self.get_out_dir().as_ref().unwrap().display()
        );
    }

    fn set_cargo_lib_name(&self) {
        std::fs::OpenOptions::new()
            .append(true)
            .write(true)
            .open("./log.txt")
            .unwrap()
            .write(
                format!(
                    "cargo:rustc-link-lib=dylib={}\n",
                    self.get_lib_name().display()
                )
                .as_bytes(),
            );
        println!(
            "cargo:rustc-link-lib=dylib={}",
            self.get_lib_name().display()
        );
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

    pub fn finish(self) {
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
            .arg("build-lib")
            .arg("-dynamic")
            .arg(format!(
                "-femit-bin={}.{}",
                self.get_emit_path().display(),
                self.get_lib_ft()
            ))
            .arg("--cache-dir")
            .arg(self.get_out_dir().expect("OUT_DIR to be set"))
            .arg(self.file.as_ref().expect("lib file to be set"))
            .exec();
    }
}

enum LibType {
    Static,
    Dynamic,
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
