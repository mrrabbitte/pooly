use jwt::Claims;

use crate::models::errors::AuthError;

const EXPECTED_AUDIENCE: &str = "pooly";
const POOLY_ROLE: &str = "pooly_role";

const ADMIN: &str = "admin";
const CLIENT_SERVICE: &str = "client_service";

pub enum AuthOutcome {

    Authorised(RoleToken),
    Unauthorised

}

pub enum RoleToken {

    Admin(AdminToken, Role),
    ClientService(ClientServiceToken, Role),

}

impl RoleToken {

    pub fn get_role(&self) -> &Role {
        match self {
            RoleToken::Admin(_, role) => role,
            RoleToken::ClientService(_, role) => role
        }
    }

}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Role {

    Admin,
    ClientService

}

pub struct ClientServiceToken {

    client_id: String

}

pub struct AdminToken {

    admin_id: String

}

impl TryFrom<&Claims> for RoleToken {
    type Error = AuthError;

    fn try_from(value: &Claims) -> Result<Self, Self::Error> {
        let role_maybe = value.private
            .get(POOLY_ROLE)
            .and_then(|v| v.as_str());

        let id_maybe = &value.registered.subject;

        match (role_maybe, id_maybe) {
            (Some(ADMIN), Some(admin_id)) =>
                Ok(RoleToken::Admin(
                    AdminToken { admin_id: admin_id.clone() },
                    Role::Admin )),
            (Some(CLIENT_SERVICE), Some(client_id)) =>
                Ok(RoleToken::ClientService(
                    ClientServiceToken { client_id: client_id.clone() },
                    Role::ClientService )),
            (_, _) => Err(AuthError::InvalidClaims)
        }
    }
}
