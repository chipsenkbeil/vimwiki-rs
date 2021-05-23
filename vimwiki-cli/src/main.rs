use structopt::StructOpt;
use vimwiki::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "vimwiki", bin_name = "vimwiki")]
struct Opt {}

fn main() {
    let opt = Opt::from_args();
}
