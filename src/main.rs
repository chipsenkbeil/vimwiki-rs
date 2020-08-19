use vimwiki::parse_vimwiki_str;

#[tokio::main]
async fn main() {
    let input = "= Some Header =";

    println!("{:?}", parse_vimwiki_str(input));
}
