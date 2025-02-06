use clap::Parser;

#[derive(Parser)]
#[command(name = "Conv")]
pub struct ConvCli {
    #[arg(short, long)]
    pub input: String,

    #[arg(short, long)]
    pub output:String,
}