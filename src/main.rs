use vimwiki::{Parser, VimwikiParser};

#[tokio::main]
async fn main() {
    let input = "= Some Header =\n=Another Header=\n=Third Header=";

    println!("{:?}", VimwikiParser::parse_str(input));
}
