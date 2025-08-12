mod sim;
mod render;
mod preview;
mod config;
mod avi;
mod yuv4mpeg2;

use std::io;
use std::io::Write;

struct Subcommand {
    name: &'static str,
    description: &'static str,
    run: fn(program_name: &str),
}

const SUBCOMMANDS: &[Subcommand] = &[
    Subcommand {
        name: "render",
        description: "render the final video and audio files",
        run: |_program_name| render::main().unwrap(),
    },
    Subcommand {
        name: "preview",
        description: "preview the video and audio",
        run: |_program_name| preview::main(),
    },
    Subcommand {
        name: "avi",
        description: "experiment with avi format",
        run: |_program_name| avi::main().unwrap(),
    },
    Subcommand {
        name: "help",
        description: "print this help message to stdout and exit with 0 code",
        run: |program_name| print_help(&mut std::io::stdout(), program_name).unwrap(),
    },
];

fn print_help(output: &mut impl Write, program_name: &str) -> io::Result<()> {
    writeln!(output, "Usage: {program_name} <SUBCOMMAND> [OPTIONS]")?;
    writeln!(output, "SUBCOMMANDS:")?;
    let width = SUBCOMMANDS.iter().map(|c| c.name.len()).max().unwrap_or(0);
    for Subcommand {name, description, ..} in SUBCOMMANDS {
        writeln!(output, "    {name:width$}        {description}")?;
    }
    Ok(())
}

fn main() -> Result<(), ()> {
    let mut args = std::env::args();
    let program_name = args.next().expect("Expected program name");
    let Some(subcommand_name) = args.next() else {
        print_help(&mut std::io::stderr(), &program_name).unwrap();
        eprintln!("ERROR: subcommand expected");
        std::process::exit(1);
    };
    let Some(subcommand) = SUBCOMMANDS.iter().find(|subcommand| subcommand.name == subcommand_name) else {
        print_help(&mut std::io::stderr(), &program_name).unwrap();
        eprintln!("ERROR: unknown subcommand: {}", subcommand_name);
        std::process::exit(1);
    };
    (subcommand.run)(&program_name);
    Ok(())
}
