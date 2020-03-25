use clap::{arg_enum, value_t, App, Arg};
use image_encryption::{decrypt_image, encrypt_image, load_image, write_image};

fn main() {
    let matches = App::new("image encryption program")
        .about(
            "given an image, the program encrypts/decrypts it using the given numeric key\n\
             the result is saved using the same image format as the input file\n\
             if no output file is given, the original image is overwritten",
        )
        .arg(
            Arg::with_name("mode")
                .required(true)
                .case_insensitive(true)
                .possible_values(&Mode::variants()),
        )
        .arg(Arg::with_name("key").required(true))
        .arg(Arg::with_name("input").required(true))
        .arg(Arg::with_name("output"))
        .get_matches();

    let mode = value_t!(matches, "mode", Mode).unwrap_or_else(|e| e.exit());
    let input = matches.value_of("input").unwrap();
    let output = matches.value_of("output").unwrap_or(input);
    let key = match matches.value_of("key").unwrap().parse::<i128>() {
        Ok(number) => number,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    let mut img = match load_image(input) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    match mode {
        Mode::Enc => encrypt_image(&mut img, key),
        Mode::Dec => decrypt_image(&mut img, key),
    }

    if let Err(err) = write_image(output, img) {
        eprintln!("{}", err)
    };
}

arg_enum! {
    #[derive(PartialEq, Debug)]
    pub enum Mode {
        Enc, Dec
    }
}
