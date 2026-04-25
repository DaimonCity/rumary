use uuid::Uuid;
use crate::domain::launcher::Profile;

pub struct Client {
    pub id: Uuid,
    pub profiles: Vec<Profile>
}