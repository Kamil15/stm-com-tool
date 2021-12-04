use structopt::StructOpt;

#[cfg(target_os = "linux")]
const DEFAULT_COM: &str = "NON";

#[cfg(target_os = "windows")]
const DEFAULT_COM: &str = "COM1";
#[derive(Debug, StructOpt, Clone)]
#[structopt(name = "COM_tool", about = "About.")]
pub struct ArgCommands {
    #[structopt(short, long, default_value = DEFAULT_COM)]
    pub target: String,

    #[structopt(short, long)]
    pub baud_rate: Option<u32>,

    #[structopt(subcommand)]
    pub mode: ArgMode,

    #[structopt(short, long)]
    pub verbose: bool,

    #[structopt(short, long)]
    pub clear_screen: bool,
}

#[derive(Debug, StructOpt, Clone)]
pub enum ArgMode {
    Interactive,
    Loopshot {
        #[structopt(short, long)]
        interval: u64,

        #[structopt(short, long)]
        command: String,
    },
    Oneshot {
        #[structopt(short, long)]
        command: String,
    },
}
