use vimwiki::{Parser, VimwikiParser};

#[tokio::main]
async fn main() {
    let input = "= Some Header =";

    println!("{:?}", VimwikiParser::parse_str(input));
}
