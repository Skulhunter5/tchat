use tchat::Client;

fn main() {
    let client = Client::new();
    client.run().unwrap();
}
