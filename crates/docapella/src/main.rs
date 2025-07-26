use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

use docapella::commands::build::{run as build, BuildArgs};
use docapella::commands::init::{run as init, InitArgs};

#[derive(Parser, Debug, Clone)]
#[command(about = "Docapella, a documentation generator", long_about = None)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(long, global = true, default_value = "auto")]
    color: Color,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
#[clap(rename_all = "lowercase")]
enum Color {
    Always,
    Auto,
    Never,
}

impl Color {
    fn init(self) {
        // Set a supports-color override based on the variable passed in.
        match self {
            Color::Always => owo_colors::set_override(true),
            Color::Auto => {}
            Color::Never => owo_colors::set_override(false),
        }
    }
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Create a new project. Defaults to the current directory.
    Init {
        #[arg(default_value = ".")]
        working_dir: PathBuf,
    },
    /// Build your documentation and create a publishable bundle
    Build {
        #[arg(default_value = ".")]
        working_dir: PathBuf,
    },
    /// Run a local server to preview your documentation
    Dev {
        #[arg(default_value = ".")]
        working_dir: PathBuf,
    },
}

fn main() {
    let args = Args::parse();
    args.color.init();

    let mut stdout = std::io::stdout();

    let result = match args.command {
        Some(Commands::Init { working_dir }) => init(InitArgs {
            working_dir: &working_dir,
            title: None,
            stdout: &mut stdout,
        }),
        Some(Commands::Build { working_dir }) => build(BuildArgs {
            out_dir: working_dir.join("_build"),
            working_dir,
            stdout: &mut stdout,
        }),
        Some(Commands::Dev { .. }) => {
            todo!()
        }
        None => {
            Args::command().print_help().unwrap();
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        println!("{:?}", e);
        std::process::exit(1);
    }
}
