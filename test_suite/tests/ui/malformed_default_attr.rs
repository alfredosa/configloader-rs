use configloader::ConfigLoader;

#[derive(ConfigLoader)]
struct Config {
    #[default(8080)]
    port: u16,
}

fn main() {}
