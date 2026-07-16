mod greeting;

const MAX_POSITIONAL_ARGUMENTS: usize = 1;
const EXIT_USAGE: i32 = 64;
const EXIT_RENDER: i32 = 65;

fn select_name(arguments: &[String]) -> Result<Option<&str>, &'static str> {
    if arguments.len() > MAX_POSITIONAL_ARGUMENTS {
        return Err("too many positional arguments");
    }
    Ok(arguments.first().map(String::as_str))
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let name = match select_name(&arguments) {
        Ok(name) => name,
        Err(message) => {
            eprintln!("usage: schema-rust-app [name]: {message}");
            std::process::exit(EXIT_USAGE);
        }
    };
    match greeting::render(name) {
        Ok(message) => println!("{message}"),
        Err(message) => {
            eprintln!("generated greeting failed: {message}");
            std::process::exit(EXIT_RENDER);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::select_name;

    #[test]
    fn accepts_zero_or_one_name() {
        assert_eq!(select_name(&[]).unwrap(), None);
        assert_eq!(select_name(&["Mantle".to_string()]).unwrap(), Some("Mantle"));
    }

    #[test]
    fn rejects_extra_names() {
        let arguments = ["first".to_string(), "second".to_string()];
        assert_eq!(select_name(&arguments).unwrap_err(), "too many positional arguments");
    }
}
