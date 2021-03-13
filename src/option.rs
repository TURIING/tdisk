use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
name = "tdisk",
author = "turiing",
about = "A convenient private network disk.")]
pub struct AppArgs {
    #[structopt(subcommand)]
    pub up: Option<Sub>,
}

#[derive(Debug, StructOpt)]
pub enum Sub {
    Up {
        #[structopt(short,long)]
        Src: String,
        #[structopt(short,long)]
        Dec: String,
    },
    Down {
        #[structopt(short,long)]
        Src: String,
        #[structopt(short,long)]
        Dec: Option<String>,
    }
}




