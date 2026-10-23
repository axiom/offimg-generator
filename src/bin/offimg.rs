use std::{path::PathBuf, process::ExitCode};

use clap::{Args, Parser, Subcommand};
use offimg_generator::{
    Result,
    dither::Algorithm,
    export,
    pipeline::{Framing, Settings, Source},
};

#[derive(Parser)]
#[command(
    version,
    about = "Convert images to 400×240, 1-bit R47 OFF-image BMP files"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Convert one PNG, JPEG, BMP, or WebP image without a desktop
    Convert(Convert),
}

#[derive(Args)]
struct Convert {
    /// Source image (never overwritten)
    input: PathBuf,
    /// Destination BMP (created only after successful conversion)
    output: PathBuf,
    /// Framing: fit (white margins) or crop (center-crop)
    #[arg(long, default_value = "fit")]
    fit: Framing,
    /// Dithering algorithm: threshold
    #[arg(long, default_value = "threshold")]
    dither: Algorithm,
    /// Replace an existing destination, but never the source
    #[arg(long)]
    force: bool,
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Convert(args) => {
            let settings = Settings {
                framing: args.fit,
                algorithm: args.dither,
                ..Settings::default()
            };
            let source = Source::open(&args.input)
                .map_err(|error| format!("cannot load {}: {error}", args.input.display()))?;
            let mono = source.convert(&settings)?;
            export::save(&mono, &args.input, &args.output, args.force)
                .map_err(|error| format!("cannot export {}: {error}", args.output.display()))?;
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
