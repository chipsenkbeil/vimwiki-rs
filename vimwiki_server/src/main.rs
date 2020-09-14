use std::convert::TryInto;
use vimwiki_server::{components::Page, RawStr, LC};

#[tokio::main]
async fn main() {
    let input =
        RawStr::Vimwiki("= Some Header =\n=Another Header=\n=Third Header=");
    let page: LC<Page> = input.try_into().expect("Failed to parse input");

    println!("{:?}", page);
}
