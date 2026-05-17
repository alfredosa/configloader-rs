use configloader::ConfigLoader;

#[derive(ConfigLoader)]
struct Config {
    #[env(PORT)]
    port: u16,
}

fn main() {}
