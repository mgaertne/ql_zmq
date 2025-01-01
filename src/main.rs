mod cmd_line;
mod linefeed_helper;
mod zmq_helper;

use std::{sync::atomic::AtomicBool, thread};

use anyhow::Result;

use crossbeam_channel::unbounded;

use clap::Parser;

use linefeed_helper::run_terminal;
use zmq_helper::run_zmq;

use cmd_line::CommandLineOptions;

pub(crate) static CONTINUE_RUNNING: AtomicBool = AtomicBool::new(true);

fn main() -> Result<()> {
    let args = CommandLineOptions::parse();

    let (zmq_sender, zmq_receiver) = unbounded::<String>();
    let (display_sender, display_receiver) = unbounded::<String>();

    let cloned_args = args.clone();

    thread::scope(|scope| {
        println!("Ctrl-C or 'exit' to exit rcon session");

        let zmq_thread = scope.spawn(move || run_zmq(cloned_args, zmq_receiver, display_sender));

        run_terminal(args, zmq_sender, display_receiver)?;

        zmq_thread.join().unwrap()
    })?;

    Ok(())
}
