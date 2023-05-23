use std::env;
use std::{path::Path, process::exit};
use swc_common::{
    self,
    errors::{ColorConfig, Handler},
    sync::Lrc,
    SourceMap,
};
use swc_ecma_ast::Module;
use swc_ecma_codegen::Emitter;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Must pass input filename");
        exit(-1);
    }
    let filename = &args[1];

    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    let fm = cm
        .load_file(Path::new(filename))
        .expect("failed to load input file");

    let lexer = Lexer::new(
        Syntax::Es(Default::default()),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut p = Parser::new_from(lexer);
    let res = p
        .parse_module()
        .map_err(|e| e.into_diagnostic(&handler).emit());

    for e in p.take_errors() {
        e.into_diagnostic(&handler).emit()
    }

    match res {
        Ok(m) => {println!("{}", print(&m, cm)) }
        Err(_) => {}
    }
}

fn print(module: &Module, cm: Lrc<SourceMap>) -> String {
    let mut buf = vec![];
    {
        let mut emitter = Emitter {
            cfg: Default::default(),
            cm: cm.clone(),
            wr: Box::new(swc_ecma_codegen::text_writer::JsWriter::new(
                cm.clone(),
                "\n",
                &mut buf,
                None,
            )),
            comments: None,
        };

        emitter.emit_module(module).unwrap();
    }

    let s = String::from_utf8_lossy(&buf);
    s.to_string()
}
