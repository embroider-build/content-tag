use swc_common::{
    self,
    errors::{ColorConfig, Handler},
    sync::Lrc,
    SourceMap,
};
use std::{path::Path, process::exit};
use std::env;
use swc_ecma_parser::{lexer::Lexer, Capturing, Parser, StringInput, Syntax};

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

    let capturing = Capturing::new(lexer);

    let mut parser = Parser::new_from(capturing);

    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    let _module = parser
        .parse_module()
        .map_err(|e| e.into_diagnostic(&handler).emit())
        .expect("Failed to parse module.");

    println!("Tokens: {:?}", parser.input().take());
}
