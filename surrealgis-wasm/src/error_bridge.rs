/// Convert any domain error implementing std::error::Error into a String.
pub fn domain_error_to_string(err: impl std::error::Error) -> String {
    err.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_io_error() {
        let err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let s = domain_error_to_string(err);
        assert_eq!(s, "file not found");
    }

    #[test]
    fn converts_function_error() {
        let err = surrealgis_functions::FunctionError::InvalidArgument("bad arg".to_string());
        let s = domain_error_to_string(err);
        assert!(s.contains("bad arg"));
    }
}
