use std::{path::PathBuf, process::ExitCode};

use clap::{Args, Parser, Subcommand, ValueEnum};
use gamedacctl::{
    BreatheDuration, BreatheMode, Color, LightingPlan, Zone,
    capture::extract_plan,
    transport::{HidTransport, Transport},
};

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Control original GameDAC and Arctis Pro lighting on Linux"
)]
struct Cli {
    /// Print the exact reports without opening a HID device.
    #[arg(long, global = true)]
    dry_run: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Set one or more verified steady-color zones.
    Static(StaticArgs),
    /// Disable illumination by setting selected zones to verified steady black.
    Off {
        #[arg(long, value_enum, default_value_t = OffTarget::Earcups)]
        target: OffTarget,
    },
    /// Generate a single-color Breathe or connected Sweep effect.
    Breathe {
        #[arg(long, value_name = "RRGGBB")]
        color: Color,
        #[arg(long, value_name = "SECONDS")]
        seconds: u16,
        #[arg(long, value_enum, default_value_t = BreatheModeArg::Synchronized)]
        mode: BreatheModeArg,
        /// Reverse the connected direction; valid only with --mode sweep.
        #[arg(long)]
        reverse: bool,
    },
    /// Replay exact, complete lighting reports from an immutable capture.
    Replay {
        #[arg(long)]
        pcap: PathBuf,
        #[arg(long, required = true, num_args = 1.., value_delimiter = ',')]
        frames: Vec<u32>,
    },
}

#[derive(Debug, Args)]
struct StaticArgs {
    #[arg(long, value_name = "RRGGBB")]
    left: Option<Color>,
    #[arg(long, value_name = "RRGGBB")]
    right: Option<Color>,
    #[arg(long, value_name = "RRGGBB")]
    microphone_live: Option<Color>,
    #[arg(long, value_name = "RRGGBB")]
    microphone_muted: Option<Color>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OffTarget {
    Earcups,
    Microphone,
    All,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum BreatheModeArg {
    Synchronized,
    Sweep,
}

impl From<BreatheModeArg> for BreatheMode {
    fn from(value: BreatheModeArg) -> Self {
        match value {
            BreatheModeArg::Synchronized => Self::Synchronized,
            BreatheModeArg::Sweep => Self::Sweep,
        }
    }
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("gamedacctl: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let plan = match cli.command {
        Command::Static(args) => LightingPlan::steady(
            [
                (Zone::Left, args.left),
                (Zone::Right, args.right),
                (Zone::MicrophoneLive, args.microphone_live),
                (Zone::MicrophoneMuted, args.microphone_muted),
            ]
            .into_iter()
            .filter_map(|(zone, color)| color.map(|color| (zone, color))),
        )?,
        Command::Off { target } => LightingPlan::steady(match target {
            OffTarget::Earcups => vec![(Zone::Left, Color::BLACK), (Zone::Right, Color::BLACK)],
            OffTarget::Microphone => vec![
                (Zone::MicrophoneLive, Color::BLACK),
                (Zone::MicrophoneMuted, Color::BLACK),
            ],
            OffTarget::All => vec![
                (Zone::Left, Color::BLACK),
                (Zone::Right, Color::BLACK),
                (Zone::MicrophoneLive, Color::BLACK),
                (Zone::MicrophoneMuted, Color::BLACK),
            ],
        })?,
        Command::Breathe {
            color,
            seconds,
            mode,
            reverse,
        } => LightingPlan::breathe(
            color,
            BreatheDuration::from_seconds(seconds)?,
            mode.into(),
            reverse,
        )?,
        Command::Replay { pcap, frames } => extract_plan(&pcap, &frames)?,
    };

    if cli.dry_run {
        print_plan(&plan);
        return Ok(());
    }

    HidTransport::open()?.execute(&plan)?;
    let zones = plan
        .features()
        .iter()
        .map(|report| report.zone().label())
        .collect::<Vec<_>>()
        .join(", ");
    println!("Applied GameDAC lighting configuration to {zones}");
    Ok(())
}

fn print_plan(plan: &LightingPlan) {
    for report in plan.features() {
        println!(
            "feature zone={} length={} bytes={} ...",
            report.zone().label(),
            report.bytes().len(),
            hex_prefix(report.bytes(), 28)
        );
    }
    for report in plan.outputs() {
        println!(
            "output length={} bytes={} ...",
            report.bytes().len(),
            hex_prefix(report.bytes(), 8)
        );
    }
    println!("zone-mask=0x{:02X}", plan.zone_mask());
}

fn hex_prefix(bytes: &[u8], count: usize) -> String {
    bytes
        .iter()
        .take(count)
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}
