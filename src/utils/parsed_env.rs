use std::env;

pub fn get<S>(name: S, default: u64) -> u64
where
    S: Into<String>,
{
    let name_string = name.into();

    match env::var(name_string.clone()) {
        Ok(value) => match value.as_str() {
            "-1" => default,
            _ => match value.parse() {
                Ok(max) => max,
                Err(e) => {
                    eprintln!(
                        "Failed to parse environment variable '{}' as an `u64`: {}",
                        name_string, e
                    );

                    default
                }
            },
        },
        Err(_) => default,
    }
}
