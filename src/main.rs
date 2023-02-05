use std::io::ErrorKind;

use env_logger::Env;
use log::{error, info, warn};
use rustyline::error::ReadlineError;
use rustyline::{Editor, Helper};

mod reader;
use crate::reader::Tokenizer;

const HISTORY_PATH: &str = ".mal-editor";

fn main() {
    let input = "(form 123 ~@[no vabbe] (predicate? '(123)))".to_string();
    let mut tokenizer = Tokenizer::new(&input);
    while let Ok(token) = tokenizer.next() {
        print!("{:?} ", token);
    }
    panic!("Done {:?}", tokenizer.rest());

    setup_logger();
    let mut editor = Editor::<()>::new().unwrap();
    load_command_history(&mut editor);
    read_eval_print_loop(&mut editor);
    save_command_history(&mut editor);
}

fn setup_logger() {
    let env = Env::default().default_filter_or("mal_rust");
    env_logger::Builder::from_env(env).init();
}

fn load_command_history<H: Helper>(editor: &mut Editor<H>) {
    match editor.load_history(HISTORY_PATH) {
        Ok(_) => info!("loaded previous command history"),
        Err(ReadlineError::Io(io_error)) if io_error.kind() == ErrorKind::NotFound => {
            warn!("no previous command history file found")
        }
        Err(err) => error!("could not load command history\n{:?}", err),
    }
}

fn save_command_history<H: Helper>(editor: &mut Editor<H>) {
    match editor.save_history(HISTORY_PATH) {
        Ok(_) => info!("saved command history"),
        Err(err) => error!("could not save command history\n{:?}", err),
    }
}

fn read_eval_print_loop<H: Helper>(editor: &mut Editor<H>) {
    let prompt = "user> ";
    loop {
        match editor.readline(prompt) {
            Ok(line) if !line.is_empty() => {
                editor.add_history_entry(&line);
                println!("{line}");
            }
            Ok(_) => continue,
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                error!("error reading line\n{:?}", err);
                break;
            }
        }
    }
}
