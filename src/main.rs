mod cmd_line;
mod linefeed_helper;
mod zmq_helper;

use core::sync::atomic::AtomicBool;

use async_std::{channel::unbounded, task};

use anyhow::Result;

use clap::Parser;

use linefeed_helper::run_terminal;
use zmq_helper::run_zmq;

use cmd_line::CommandLineOptions;

extern crate alloc;

pub(crate) static CONTINUE_RUNNING: AtomicBool = AtomicBool::new(true);

#[async_std::main]
async fn main() -> Result<()> {
    let args = CommandLineOptions::parse();

    let (zmq_sender, zmq_receiver) = unbounded::<String>();
    let (display_sender, display_receiver) = unbounded::<String>();

    let cloned_args = args.clone();

    println!("Ctrl-C or 'exit' to exit rcon session");

    let zmq_task = task::spawn(run_zmq(cloned_args, zmq_receiver, display_sender));
    let terminal_task = task::spawn(run_terminal(args, zmq_sender, display_receiver));

    terminal_task.await?;
    zmq_task.cancel().await;

    Ok(())
}
