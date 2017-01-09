// Crates ---------------------------------------------------------------------
extern crate server;
extern crate clap;


// Server Runnable ------------------------------------------------------------
fn main() {

    let matches = clap::App::new("Shooter Server")
        .version("0.1")
        .author("Ivo Wetzel <ivo.wetzel@googlemail.com>")
        .about("A shooter game")
        .arg(clap::Arg::with_name("addr")
            .short("a")
            .long("addr")
            .value_name("LISTENING_ADDR")
            .help("Specifies the host:port on which to listen for network traffic.")
            .takes_value(true)
            .default_value("0.0.0.0:7156")
        )
        .get_matches();

    server::run(
        matches.value_of("addr").unwrap().to_string()

    ).join().ok();

}

