extern crate fastboot;
use fastboot::fastboot::Fastboot;
use fastboot::usbio;

extern crate getopts;
use getopts::Options;

fn usage(program: &str, opts: &Options) {
    let brief = format!(
        "Version: {}\nUsage: {} [options]",
        env!("CARGO_PKG_VERSION"),
        program
    );
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("h", "help", "Print help");
    opts.optopt("", "vid", "Vendor ID", "<hex>");
    opts.optopt("", "pid", "Product ID", "<hex>");
    opts.optopt("", "size", "Size to download", "<size>");

    if args.len() <= 1 {
        usage(&program, &opts);
        return;
    }
    let matches = opts.parse(&args[1..]).unwrap_or_else(|err| {
        eprintln!("{} failed to parse arguments ({})!", &program, err);
        usage(&program, &opts);
        std::process::exit(-1);
    });

    if matches.opt_present("h") {
        usage(&program, &opts);
        std::process::exit(0);
    }

    let vid = match matches.opt_str("vid") {
        Some(value) => u16::from_str_radix(&value, 16).expect("Parsing vendor ID failed"),
        None => 0x0451,
    };
    let pid = match matches.opt_str("pid") {
        Some(value) => u16::from_str_radix(&value, 16).expect("Parsing product ID failed"),
        None => 0xd022,
    };

    let size = match matches.opt_str("size") {
        Some(value) => usize::from_str_radix(&value, 10).expect("Parsing size failed"),
        None => 512,
    };

    let context = usbio::UsbContext::new();
    let mut device = context
        .open(vid, pid)
        .expect(&format!("Failed to open {}:{}", vid, pid));

    let data = vec![0; size];
    println!("{:?}", device.download(&data));
}
