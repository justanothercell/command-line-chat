use serde::{Deserialize, Serialize};

pub mod protocol;
pub mod validator;

pub fn serialize<T: Serialize>(value: &T) -> Result<String, String>{
    match serde_json::ser::to_string(value) {
        Ok(s) => Ok(s),
        Err(e) => Err(e.to_string())
    }
}

pub fn deserialize<T: for<'a> Deserialize<'a>>(s: &str) -> Result<T, String>{
    match serde_json::de::from_str(s) {
        Ok(v) => Ok(v),
        Err(e) => Err(e.to_string())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tesu() {

    }
}
