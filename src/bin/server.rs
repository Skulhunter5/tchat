use tchat::Server;

fn main() {
    let server = Server::new();
    server.run().unwrap();
}
