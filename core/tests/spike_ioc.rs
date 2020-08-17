use std::sync::Arc;

trait Factory<T> {
    fn get(&self) -> T;
}

struct Container{}


pub struct BeanA {
    pub bean_b: Arc<BeanB>
}

impl Factory<Arc<BeanA>> for Container {
    fn get(&self) -> Arc<BeanA> {
        BeanA {
            bean_b: self.get()
        }.into()
    }
}

pub struct BeanB {
    // Enabling this, the code compiles, but it throws a stackoverflow at runtime. No bueno...
    //bean_a: Arc<BeanA>
}

impl Factory<Arc<BeanB>> for Container {
    fn get(&self) -> Arc<BeanB> {
        BeanB{
            //bean_a: self.get()
        }.into()
    }
}

fn do_smt(_a: Arc<BeanA>, _b: Arc<BeanB>) {}

#[test]
fn start() {
    let container = Container{};
    do_smt(container.get(), container.get());
}