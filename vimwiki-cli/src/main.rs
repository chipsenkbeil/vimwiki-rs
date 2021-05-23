use structopt::StructOpt;
use vimwiki::*;

#[derive(Debug, StructOpt)]
struct Opt {}

fn main() {
    let opt = Opt::from_args();
}
