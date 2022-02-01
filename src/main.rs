pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input_file = &args[1];

    match rust_coding_test::utils::process_tx(input_file) {
        Ok(_) => {}
        Err(err) => {
            println!("The program has failed! The following error occurred -> {}", err.to_string())
        }
    }
}

