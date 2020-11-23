use rand::distributions::Alphanumeric;
use rand::Rng;

pub struct RandomService {}

impl RandomService {
    #[inline]
    pub fn random_string(length: usize) -> String {
        rand::thread_rng().sample_iter(&Alphanumeric).take(length).collect::<String>()
    }

    #[inline]
    pub fn random_numeric_string(digits: u32) -> String {
        let max = (10usize).pow(digits);
        let number = rand::thread_rng().gen_range(0, max);
        format!("{:0width$}", number, width = (digits as usize))
    }
}

#[cfg(test)]
pub mod test {

    use super::*;

    #[test]
    fn should_return_a_random_string() {
        assert_eq!(10, RandomService::random_string(10).len());
        assert_eq!(0, RandomService::random_string(0).len());
    }

    #[test]
    fn should_generate_a_random_number_of_6_digits() {
        let mut used = vec![];
        for _ in 0..100 {
            let code = RandomService::random_numeric_string(6);
            //println!("Generated code: {}", code);
            assert_eq!(6, code.len());
            assert!(!used.contains(&code));
            used.push(code);
        }
    }

    #[test]
    fn should_generate_a_random_number_of_fixed_digits() {
        for _ in 0..10000 {
            let digits = rand::thread_rng().gen_range(1, 20);
            let code = RandomService::random_numeric_string(digits);
            //            println!("Generated code: {}", code);
            assert_eq!(digits as usize, code.len());
            assert!(code.chars().all(char::is_numeric));
        }
    }
}
