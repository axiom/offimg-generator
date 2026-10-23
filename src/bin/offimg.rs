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
    #[arg(long, default_value_t = Settings::default().framing)]
    fit: Framing,
    /// Dithering algorithm: threshold, floyd-steinberg, or bayer
    #[arg(long, default_value_t = Settings::default().algorithm)]
    dither: Algorithm,
    /// Brightness offset (-1 to 1)
    #[arg(long, allow_hyphen_values = true, default_value_t = Settings::default().brightness)]
    brightness: f32,
    /// Contrast around middle gray (0 to 4)
    #[arg(long, allow_hyphen_values = true, default_value_t = Settings::default().contrast)]
    contrast: f32,
    /// Gamma correction (0.1 to 4); greater than 1 brightens
    #[arg(long, allow_hyphen_values = true, default_value_t = Settings::default().gamma)]
    gamma: f32,
    /// Cutoff for threshold dithering (0 to 255); values at the cutoff are white
    #[arg(long, default_value_t = Settings::default().threshold)]
    threshold: u8,
    /// Swap black and white after dithering
    #[arg(long)]
    invert: bool,
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
                brightness: args.brightness,
                contrast: args.contrast,
                gamma: args.gamma,
                threshold: args.threshold,
                invert: args.invert,
            };
            settings.validate()?;
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
