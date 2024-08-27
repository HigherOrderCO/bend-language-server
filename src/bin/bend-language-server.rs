use std::env;

use tokio::io::AsyncWriteExt;
use tower_lsp::{LspService, Server};

use bend_language_server::server::*;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();

    if env::args().any(|arg| arg == "--version") {
        let version = format!("{}\n", env!("CARGO_PKG_VERSION"));
        stdout
            .write_all(version.as_bytes())
            .await
            .expect("Couldn't write to stdout.");
        return;
    }

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
