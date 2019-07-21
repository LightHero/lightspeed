use crate::test;

#[test]
fn should_instantiate_db_service() {
    test(|auth_module| {
        auth_module.db
    });
}