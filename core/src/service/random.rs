use rand::RngExt;
use rand::distr::Alphanumeric;

pub struct LsRandomService {}

impl LsRandomService {
    /// Returns a string of exactly `length` alphanumeric characters
    #[inline]
    pub fn random_string(length: usize) -> String {
        rand::rng().sample_iter(&Alphanumeric).take(length).map(char::from).collect::<String>()
    }

    /// Returns a string of exactly `length` decimal digits
    #[inline]
    pub fn random_numeric_string(length: usize) -> String {
        let mut rng = rand::rng();
        (0..length).map(|_| char::from(b'0' + rng.random_range(0..10u8))).collect()
    }
}

#[cfg(test)]
pub mod test {

    use super::*;

    #[test]
    fn should_return_a_random_string() {
        assert_eq!(10, LsRandomService::random_string(10).len());
        assert_eq!(0, LsRandomService::random_string(0).len());
    }

    #[test]
    fn should_generate_a_random_number_of_8_digits() {
        let mut used = vec![];
        for _ in 0..100 {
            let code = LsRandomService::random_numeric_string(8);
            assert_eq!(8, code.len());
            assert!(!used.contains(&code));
            used.push(code);
        }
    }

    #[test]
    fn should_generate_a_random_number_of_fixed_digits() {
        for _ in 0..10000 {
            let digits = rand::rng().random_range(1..20usize);
            let code = LsRandomService::random_numeric_string(digits);
            assert_eq!(digits, code.len());
            assert!(code.chars().all(char::is_numeric));
        }
    }

    #[test]
    fn should_not_panic_for_long_lengths() {
        for length in [20usize, 38, 100, 1024] {
            let code = LsRandomService::random_numeric_string(length);
            assert_eq!(length, code.len(), "wrong length for {length}");
            assert!(code.chars().all(char::is_numeric), "non-numeric output: {code}");
        }
    }

    #[test]
    fn should_handle_zero_length() {
        assert_eq!("", LsRandomService::random_numeric_string(0));
    }
}
