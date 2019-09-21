use crate::repository::SchemaContentMappingRepository;

#[derive(Clone)]
pub struct SchemaContentMappingService<SchemaContentMappingRepo: SchemaContentMappingRepository> {
    schema_content_repo: SchemaContentMappingRepo,
}

impl<SchemaContentMappingRepo: SchemaContentMappingRepository> SchemaContentMappingService<SchemaContentMappingRepo> {
    pub fn new(schema_content_repo: SchemaContentMappingRepo) -> Self {
        SchemaContentMappingService {
            schema_content_repo,
        }
    }

}

#[cfg(test)]
mod test {

}
