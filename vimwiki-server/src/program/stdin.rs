use crate::{graphql, Opt};
use log::{error, info};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Input {
    /// Id of incoming payload to use when sending back a response
    id: usize,

    /// Payload as serialized
    payload: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Output {
    /// Id of outgoing payload
    id: usize,

    /// Payload as serialized JSON string
    payload: String,
}

/// Spawns a worker to process stdin and communicate back over stdout & stderr
pub async fn run(_opt: Opt) {
    let schema = graphql::new_schema();

    info!("Monitoring stdin...");
    // NOTE: For now, we are using std lib's stdin & stdout due to
    //       blocking limitations within tokio's implementation causing
    //       problems: https://github.com/tokio-rs/tokio/issues/2466
    let stdin = std::io::stdin();
    let mut buffer = String::new();
    loop {
        let result = stdin.read_line(&mut buffer);
        match result {
            Ok(n) if n > 0 => {
                // Parse our input if possible, in the form of
                // { "id": ..., "payload": ... }
                if let Ok(Input { id, payload }) = serde_json::from_str(&buffer)
                {
                    let response = schema.execute(&payload).await;
                    send_response(id, response).await;
                }

                buffer.clear();
            }
            Ok(_) => break,
            Err(x) => {
                error!("Failed to read stdin: {}", x);
                break;
            }
        }
    }
}

async fn send_response(id: usize, response: async_graphql::Response) {
    match serde_json::to_string(&response)
        .map(|payload| Output { id, payload })
        .and_then(|output| serde_json::to_string(&output))
    {
        Ok(msg) => println!("{}", msg),
        Err(x) => eprintln!("{}", x),
    }
}
