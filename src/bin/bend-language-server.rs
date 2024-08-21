use tower_lsp::{LspService, Server};

use bend_language_server::server::*;

// #[tokio::main]
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let empty1 = tokio::io::empty();
    let empty2 = tokio::io::empty();
    // let stdin = tokio::io::stdin();
    // let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    // Server::new(stdin, stdout, socket).serve(service).await;
    Server::new(empty1, empty2, socket).serve(service).await;
}
