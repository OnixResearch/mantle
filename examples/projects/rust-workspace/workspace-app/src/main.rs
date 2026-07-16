use std::process::ExitCode;

const USAGE_EXIT_CODE: u8 = 64;
const MAX_POSITIONAL_ARGUMENTS: usize = 1;

fn selected_name(arguments: &[String]) -> Result<&str, ()> {
    if arguments.len() > MAX_POSITIONAL_ARGUMENTS {
        return Err(());
    }

    debug_assert!(arguments.len() <= MAX_POSITIONAL_ARGUMENTS);
    debug_assert!(MAX_POSITIONAL_ARGUMENTS > 0);
    Ok(arguments.first().map(String::as_str).unwrap_or(greeting::DEFAULT_NAME))
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Ok(name) = selected_name(&arguments) else {
        eprintln!("usage: workspace-app [name]");
        return ExitCode::from(USAGE_EXIT_CODE);
    };
    let Ok(message) = greeting::render(name) else {
        eprintln!("name must not be empty");
        return ExitCode::FAILURE;
    };

    println!("{message}");
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_default_and_explicit_names() {
        assert_eq!(selected_name(&[]).unwrap(), greeting::DEFAULT_NAME);
        assert_eq!(selected_name(&["Mantle".to_string()]).unwrap(), "Mantle");
    }

    #[test]
    fn rejects_extra_arguments() {
        let arguments = ["first".to_string(), "second".to_string()];
        assert!(selected_name(&arguments).is_err());
        assert_eq!(arguments.len(), MAX_POSITIONAL_ARGUMENTS + 1);
    }
}
