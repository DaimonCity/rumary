use crate::domain::api::share_target::{ShareTarget, ShareTargetError};
use crate::domain::perms::value_object::group::{GroupName, GroupWeight};
use crate::domain::perms::value_object::user::UserId;
use crate::dto::api::request::share_target::ShareTargetRequest;

impl ShareTargetRequest {
    pub fn into_domain(self) -> Result<ShareTarget, ShareTargetError> {
        Ok(match self {
            ShareTargetRequest::Peers => ShareTarget::Peers,
            ShareTargetRequest::MinRank { weight } => {
                let weight = GroupWeight::new(weight)
                    .map_err(ShareTargetError::InvalidWeight)?;
                ShareTarget::MinRank(weight)
            }
            ShareTargetRequest::Group { name } => {
                let name = GroupName::try_from(name.as_str())
                    .map_err(ShareTargetError::InvalidGroupName)?;
                ShareTarget::Role(name)
            }
            ShareTargetRequest::Users { user_ids } => {
                ShareTarget::Users(user_ids.into_iter().map(UserId::from).collect())
            }
        })
    }
}