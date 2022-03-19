use jwt::Claims;

use crate::models::errors::AuthError;

const EXPECTED_AUDIENCE: &str = "pooly";
const POOLY_ROLE: &str = "pooly_role";

const ADMIN: &str = "admin";
const CLIENT_SERVICE: &str = "client_service";

#[derive(Debug)]
pub enum AuthOutcome {

    Authenticated(RoleToken),
    Unauthenticated

}

#[derive(Debug)]
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

    pub fn get_client_id(&self) -> &str {
        match self {
            RoleToken::Admin(_, _) => panic!("Wrong Role, should panic."),
            RoleToken::ClientService(client_id, _) => &client_id.client_id
        }
    }

}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Role {

    Admin,
    ClientService

}

#[derive(Debug)]
pub struct ClientServiceToken {

    client_id: String

}

#[derive(Debug)]
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
