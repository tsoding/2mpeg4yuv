mod sim;
mod render;
mod preview;
mod config;
mod avi;

use std::io;
use std::io::Write;

fn print_help(output: &mut impl Write, program: &str) -> io::Result<()> {
    writeln!(output, "Usage: {} <SUBCOMMAND> [OPTIONS]", program)?;
    writeln!(output, "SUBCOMMANDS:")?;
    writeln!(output, "    render        render the final video and audio files")?;
    writeln!(output, "    preview       preview the video and audio")?;
    writeln!(output, "    avi           experiment with avi format")?;
    writeln!(output, "    help          print this help message to stdout and exit with 0 code")?;
    Ok(())
}

fn main() -> Result<(), ()> {
    let mut args = std::env::args();
    let program = args.next().expect("Expected program name"); // skip program
    if let Some(subcommand) = args.next() {
        match subcommand.as_str() {
            "render" => render::main().unwrap(),
            "preview" => preview::main(),
            "avi" => avi::main()?,
            "help" => print_help(&mut std::io::stdout(), &program).unwrap(),
            _ => {
                print_help(&mut std::io::stderr(), &program).unwrap();
                eprintln!("ERROR: unknown subcommand: {}", subcommand);
                std::process::exit(1);
            }
        }
    } else {
        print_help(&mut std::io::stderr(), &program).unwrap();
        eprintln!("ERROR: subcommand expected");
        std::process::exit(1);
    }
    Ok(())
}
