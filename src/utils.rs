use crate::error::RemError;

pub fn get_seconds(time_str: &str) -> Result<u64, RemError> {
    let mut total = 0;

    for part in time_str.split_whitespace() {
        let (value_str, unit) = part.split_at(part.len() - 1);

        let value: u64 = value_str.parse().map_err(|e| {
            eprintln!("error while parsing time: {}", e);
            RemError::ParsingError
        })?;

        match unit {
            "h" => total += value * 3600,
            "m" => total += value * 60,
            "s" => total += value,
            _ => {
                eprintln!("error while parsing time: invalid time unit, possible value: h, m, s");
                return Err(RemError::ParsingError);
            },
        }
    }

    Ok(total)
}
