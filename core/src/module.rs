use crate::error::LightSpeedError;
use log::info;

pub trait ModuleBuilder<T: Module> {
    fn build(&self) -> Result<T, LightSpeedError>;
}

pub trait Module {
    fn start(&mut self) -> Result<(), LightSpeedError>;
}

pub fn start(modules: &mut [&mut dyn Module]) -> Result<(), LightSpeedError> {
    info!("Begin modules 'start' phase");
    for module in modules.iter_mut() {
        module.start()?;
    }
    Ok(())
}

#[cfg(test)]
mod test {

    use crate::LightSpeedError;
    use std::rc::Rc;
    use std::sync::Mutex;

    #[test]
    fn should_start_all() {
        let output = Rc::new(Mutex::new(vec![]));

        let mut mod1 = SimpleModOne {
            output: output.clone(),
            name: "one".to_string(),
        };
        let mut mod2 = SimpleModTwo {
            output: output.clone(),
            name: "two".to_string(),
            fail: false,
        };

        let mut modules: Vec<&mut dyn super::Module> = vec![&mut mod1, &mut mod2];

        let result = super::start(modules.as_mut());

        assert!(result.is_ok());
        assert_eq!(2, output.lock().unwrap().len());
        assert_eq!(
            &"one-start".to_string(),
            output.lock().unwrap().get(0).unwrap()
        );
        assert_eq!(
            &"two-start".to_string(),
            output.lock().unwrap().get(1).unwrap()
        );
    }

    #[test]
    fn should_fail_on_start() {
        let output = Rc::new(Mutex::new(vec![]));

        let mut mod1 = SimpleModOne {
            output: output.clone(),
            name: "one".to_string(),
        };
        let mut mod2 = SimpleModTwo {
            output: output.clone(),
            name: "two".to_string(),
            fail: true,
        };

        let mut modules: Vec<&mut dyn super::Module> = vec![&mut mod1, &mut mod2];

        let result = super::start(&mut modules);

        assert!(result.is_err());

        match result {
            Err(err) => match err {
                LightSpeedError::ModuleStartError { message } => {
                    assert_eq!("test_failure", message)
                }
                _ => assert!(false),
            },
            _ => assert!(false),
        }

        assert_eq!(1, output.lock().unwrap().len());
        assert_eq!(
            &"one-start".to_string(),
            output.lock().unwrap().get(0).unwrap()
        );
    }

    #[derive(Clone)]
    struct SimpleModOne {
        output: Rc<Mutex<Vec<String>>>,
        name: String,
    }

    impl super::Module for SimpleModOne {
        fn start(&mut self) -> Result<(), LightSpeedError> {
            let mut owned = self.name.to_owned();
            owned.push_str(&"-start");
            self.output.lock().unwrap().push(owned);
            Ok(())
        }
    }

    #[derive(Clone)]
    struct SimpleModTwo {
        output: Rc<Mutex<Vec<String>>>,
        name: String,
        fail: bool,
    }

    impl super::Module for SimpleModTwo {
        fn start(&mut self) -> Result<(), LightSpeedError> {
            if self.fail {
                return Err(LightSpeedError::ModuleStartError {
                    message: "test_failure".to_owned(),
                });
            }

            let mut owned = self.name.to_owned();
            owned.push_str(&"-start");
            self.output.lock().unwrap().push(owned);

            Ok(())
        }
    }
}
