use std::env;
use command_macro::{command_handler, command};

pub struct App {
}

#[command_handler]
impl App {
    pub fn new() -> App {
        App { }
    }

    #[command("one")]
    fn handle_one(&self, args: &[String]) {
        println!("You ran command one with args={:?}", args);
    }

    #[command("two")]
    fn handle_two(&self, args: &[String]) {
        println!("You ran command two with args={:?}", args);
    }

    pub fn run(&self, command: &str, command_args: &[String]) {
        self.handle_command(command, command_args, |c, a| todo!("command={} args={:?}", c, a))
    }
}

fn main() {
    let app = App::new();
    let args: Vec<String> = env::args().collect();
    let command: &str = &args[1];
    let command_args: &[String] = &args[2..];
    app.run(command, command_args);
}