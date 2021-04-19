#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;

use std::path::Path;
use std::process::{exit, Command};

use rustc_driver::Callbacks;
use rustc_driver::Compilation;

use rustc_hir::intravisit::{self, NestedVisitorMap, Visitor};
use rustc_interface::interface::Compiler;
use rustc_interface::Config;
use rustc_interface::Queries;
use rustc_middle::hir::map::Map;

struct MyCallback;

impl Callbacks for MyCallback {
    fn config(&mut self, _config: &mut Config) {}

    fn after_parsing<'tcx>(
        &mut self,
        _compiler: &Compiler,
        _queries: &'tcx Queries<'tcx>,
    ) -> Compilation {
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
        compiler: &Compiler,
        queries: &'tcx Queries<'tcx>,
    ) -> Compilation {
        println!("Hello from my callback!!!");
        compiler.session().abort_if_errors();
        queries.global_ctxt().unwrap().peek_mut().enter(|tcx| {
            let krate = tcx.hir().krate();

            let mut expr_visitor = ExprVisitor { tcx };
            for (_, item) in &krate.items {
                expr_visitor.visit_item(item);
            }
        });
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

    let compilation = rustc_driver::catch_with_exit_code(move || {
        let mut args = std::env::args().collect::<Vec<_>>();
        let is_wrapper_mode =
            args.get(1).map(Path::new).and_then(Path::file_stem) == Some("rustc".as_ref());
        if is_wrapper_mode {
            args.remove(1);
        }
        args.extend(vec!["--sysroot".to_string(), sysroot]);
        let mut callback = MyCallback;
        rustc_driver::RunCompiler::new(&args, &mut callback).run()
    });

    exit(compilation)
}

struct ExprVisitor<'tcx> {
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
}

impl<'tcx> Visitor<'tcx> for ExprVisitor<'tcx> {
    type Map = Map<'tcx>;

    fn nested_visit_map(&mut self) -> NestedVisitorMap<Self::Map> {
        NestedVisitorMap::OnlyBodies(self.tcx.hir())
    }

    fn visit_expr(&mut self, expr: &'tcx rustc_hir::Expr<'tcx>) {
        let tcx = self.tcx;
        let hir_id = expr.hir_id;
        if let Some(def_id) = tcx.hir().opt_local_def_id(hir_id) {
            let ty = tcx.typeck(def_id).node_type(hir_id);
            println!("{:?}: {:?}", expr, ty);
        }
    }
}
