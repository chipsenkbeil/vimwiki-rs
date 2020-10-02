use vimwiki_server::run;

#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_builder()
        // .filter_level(log::LevelFilter::Trace)
        .init();
    run().await;
}
