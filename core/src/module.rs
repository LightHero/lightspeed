use crate::error::LightSpeedError;
use log::info;

#[async_trait::async_trait]
pub trait ModuleBuilder<T: Module> {
    async fn build(&self) -> Result<T, LightSpeedError>;
}

#[async_trait::async_trait]
pub trait Module {
    async fn start(&mut self) -> Result<(), LightSpeedError>;
}

pub async fn start(modules: &mut [&mut dyn Module]) -> Result<(), LightSpeedError> {
    info!("Begin modules 'start' phase");
    for module in modules.iter_mut() {
        module.start().await?;
    }
    Ok(())
}

#[cfg(test)]
mod test {

    use crate::LightSpeedError;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn should_start_all() {
        let output = Arc::new(Mutex::new(vec![]));

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

        let result = super::start(modules.as_mut()).await;

        assert!(result.is_ok());
        assert_eq!(2, output.lock().await.len());
        assert_eq!(
            &"one-start".to_string(),
            output.lock().await.get(0).unwrap()
        );
        assert_eq!(
            &"two-start".to_string(),
            output.lock().await.get(1).unwrap()
        );
    }

    #[tokio::test]
    async fn should_fail_on_start() {
        let output = Arc::new(Mutex::new(vec![]));

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

        let result = super::start(&mut modules).await;

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

        assert_eq!(1, output.lock().await.len());
        assert_eq!(
            &"one-start".to_string(),
            output.lock().await.get(0).unwrap()
        );
    }

    #[derive(Clone)]
    struct SimpleModOne {
        output: Arc<Mutex<Vec<String>>>,
        name: String,
    }

    #[async_trait::async_trait]
    impl super::Module for SimpleModOne {
        async fn start(&mut self) -> Result<(), LightSpeedError> {
            let mut owned = self.name.to_owned();
            owned.push_str(&"-start");
            self.output.lock().await.push(owned);
            Ok(())
        }
    }

    #[derive(Clone)]
    struct SimpleModTwo {
        output: Arc<Mutex<Vec<String>>>,
        name: String,
        fail: bool,
    }

    #[async_trait::async_trait]
    impl super::Module for SimpleModTwo {
        async fn start(&mut self) -> Result<(), LightSpeedError> {
            if self.fail {
                return Err(LightSpeedError::ModuleStartError {
                    message: "test_failure".to_owned(),
                });
            }

            let mut owned = self.name.to_owned();
            owned.push_str(&"-start");
            self.output.lock().await.push(owned);

            Ok(())
        }
    }
}
