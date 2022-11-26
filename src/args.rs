pub fn parse() -> Mode {
    <Mode as clap::Parser>::parse()
}

#[derive(clap::Parser)]
#[clap(author, version, about, long_about)]
pub enum Mode {
    Left(Args),
    Right,
}

#[derive(clap::Args)]
pub struct Args {
    #[arg(short, long)]
    pub long: bool,

    #[arg(short, long)]
    pub root: bool,

    #[arg(short, long)]
    pub status: Option<i32>,

    #[arg(short, long)]
    pub jobs: bool,
}
