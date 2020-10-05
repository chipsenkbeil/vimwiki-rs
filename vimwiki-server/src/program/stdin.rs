use super::{graphql, Config, Program};
use log::{error, info};

/// Spawns a worker to process stdin and communicate back over stdout & stderr
pub async fn run(program: Program, _config: Config) {
    let schema = graphql::build_schema_with_program(program);

    info!("Monitoring stdin...");
    // NOTE: For now, we are using std lib's stdin & stdout due to
    //       blocking limitations within tokio's implementation causing
    //       problems: https://github.com/tokio-rs/tokio/issues/2466
    let stdin = std::io::stdin();
    let mut input = String::new();
    loop {
        let result = stdin.read_line(&mut input);
        match result {
            Ok(n) if n > 0 => {
                let result = schema.execute(&input).await;
                match serde_json::to_string(&result) {
                    Ok(json) => println!("{}", json),
                    Err(x) => eprintln!("{}", x),
                }
            }
            Ok(_) => break,
            Err(x) => {
                error!("Failed to read stdin: {}", x);
                break;
            }
        }
    }
}
