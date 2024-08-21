use tower_lsp::{LspService, Server};

use bend_language_server::server::*;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}

// #[cfg(target_arch = "wasm32")]
// fn main() {
//     use wasi_async_runtime::*;
//     block_on(|reactor| async {
//         let client = Client::new(reactor); // <- using an unpublished wrapper around `wasi::http`

//         let a = async {
//             let url = "https://example.com".parse().unwrap();
//             let req = Request::new(Method::Get, url);
//             let res = client.send(req).await;

//             let body = read_to_end(res).await;
//             let body = String::from_utf8(body).unwrap();
//             println!("{body}");
//         };

//         let b = async {
//             let url = "https://example.com".parse().unwrap();
//             let req = Request::new(Method::Get, url);
//             let res = client.send(req).await;

//             let body = read_to_end(res).await;
//             let body = String::from_utf8(body).unwrap();
//             println!("{body}");
//         };

//         (a, b).join().await; // concurrently await both `a` and `b`.
//     })
// }
