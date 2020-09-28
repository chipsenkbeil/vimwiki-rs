use vimwiki_server::run_server;

#[tokio::main]
async fn main() {
    run_server("http://localhost:8000").await;
}
