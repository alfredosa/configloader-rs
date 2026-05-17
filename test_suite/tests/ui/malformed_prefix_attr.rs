use configloader::ConfigLoader;

#[derive(ConfigLoader)]
#[prefix(APP)]
struct Config {
    port: u16,
}

fn main() {}
