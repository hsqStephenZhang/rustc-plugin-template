#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;

use std::path::Path;
use std::process::{exit, Command};

use rustc_driver::Callbacks;
use rustc_driver::Compilation;
use rustc_interface::interface::Compiler;
use rustc_interface::Config;
use rustc_interface::Queries;

struct MyCallback;

impl Callbacks for MyCallback {
    fn config(&mut self, _config: &mut Config) {}

    fn after_parsing<'tcx>(
        &mut self,
        _compiler: &Compiler,
        _queries: &'tcx Queries<'tcx>,
    ) -> Compilation {
        println!("Hello from my callback!!!");
        Compilation::Continue
    }

    fn after_expansion<'tcx>(
        &mut self,
        _compiler: &Compiler,
        _queries: &'tcx Queries<'tcx>,
    ) -> Compilation {
        Compilation::Continue
    }

    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &Compiler,
        queries: &'tcx Queries<'tcx>,
    ) -> Compilation {
        Compilation::Continue
    }
}

fn main() {
    rustc_driver::init_rustc_env_logger();
    rustc_driver::install_ice_hook();

    let sysroot = Command::new("rustc")
        .arg("--print=sysroot")
        .current_dir(".")
        .output()
        .unwrap();
    let sysroot = String::from_utf8_lossy(&sysroot.stdout).trim().to_string();

    let run = move || {
        let mut args = std::env::args().collect::<Vec<_>>();
        let is_wrapper_mode =
            args.get(1).map(Path::new).and_then(Path::file_stem) == Some("rustc".as_ref());
        if is_wrapper_mode {
            args.remove(1);
        }
        args.extend(vec!["--sysroot".to_string(), sysroot]);
        let mut callback = MyCallback;
        rustc_driver::RunCompiler::new(&args, &mut callback).run()
    };
    let compilation = rustc_driver::catch_with_exit_code(run);

    exit(compilation)
}
