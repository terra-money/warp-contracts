mod deploy;
mod migrate;

fn main() {
    let _ = deploy::deploy();
    let _ = migrate::migrate();
}
