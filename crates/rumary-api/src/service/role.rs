use std::sync::Arc;
use crate::error::AppError;
use crate::repo::repository::RolesRepository;

pub struct RoleService {
    role_repo: Arc<dyn RolesRepository<Error=AppError>>
}

impl RoleService {
    pub fn new(role_repo: Arc<dyn RolesRepository<Error=AppError>>) -> RoleService {
        RoleService { role_repo }
    }
    
}