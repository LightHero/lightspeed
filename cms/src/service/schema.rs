use crate::repository::SchemaRepository;

#[derive(Clone)]
pub struct SchemaService<SchemaRepo: SchemaRepository> {
    schema_repo: SchemaRepo,
}

impl<SchemaRepo: SchemaRepository> SchemaService<SchemaRepo> {
    pub fn new(schema_repo: SchemaRepo) -> Self {
        SchemaService {
            schema_repo,
        }
    }

}

#[cfg(test)]
mod test {
    
}
