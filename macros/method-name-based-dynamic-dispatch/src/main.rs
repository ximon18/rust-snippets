use std::env;
use command_macro::{command_handler};

pub struct App {
}

#[command_handler]
impl App {
    pub fn new() -> App {
        App { }
    }

    fn command_one(&self, _command_args: &[String]) {
        println!("You ran command one");
    }

    fn command_two(&self, _command_args: &[String]) {
        println!("You ran command two");
    }

    fn command_unknown(&self, command: &str, command_args: &[String]) {
        todo!("command: {:#?} args: {:#?}", command, command_args)
    }

    pub fn run(&self, command: &str, command_args: &[String]) {
        self.handle_command(command, command_args)
    }
}

fn main() {
    let app = App::new();
    let args: Vec<String> = env::args().collect();
    let command: &str = &args[1];
    let command_args: &[String] = &args[2..];
    app.run(command, command_args);
}