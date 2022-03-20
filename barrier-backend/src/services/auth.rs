use crate::structs::Gate;
use std::{collections::HashMap, ptr};

pub trait Auth {
    fn get_available_rooms(&self, username: &str, password: &str) -> Option<Vec<Gate>>;
}

pub struct LDAPAuth {
    ldap_server: String,
    ldap_base: String,
    ldap_bind: String,
    ldap_filter: Option<String>,
    gate_mappings: HashMap<String, Vec<Gate>>,
}

impl LDAPAuth {
    pub fn new(
        ldap_server: String,
        ldap_base: String,
        ldap_bind: String,
        ldap_filter: Option<String>,
        gate_mappings: HashMap<String, Vec<Gate>>,
    ) -> Self {
        LDAPAuth {
            ldap_server,
            ldap_base,
            ldap_bind,
            ldap_filter,
            gate_mappings,
        }
    }
}

impl Auth for LDAPAuth {
    fn get_available_rooms(&self, username: &str, password: &str) -> Option<Vec<Gate>> {
        let ldap = openldap::RustLDAP::new(&self.ldap_server).ok()?;

        ldap.set_option(
            openldap::codes::options::LDAP_OPT_PROTOCOL_VERSION,
            &openldap::codes::versions::LDAP_VERSION3,
        );

        ldap.set_option(
            openldap::codes::options::LDAP_OPT_X_TLS_REQUIRE_CERT,
            &openldap::codes::options::LDAP_OPT_X_TLS_DEMAND,
        );

        let bind_dn = self.ldap_bind.replace("%(username)", username);

        if ldap.simple_bind(&bind_dn, password).ok()? != 0 {
            return None;
        }

        let ldap_filter = self
            .ldap_filter
            .clone()
            .map(|f| f.replace("%(username)", username));

        // Returns a LDAPResponse, a.k.a. Vec<HashMap<String,Vec<String>>>.
        let responses = ldap
            .ldap_search(
                &self.ldap_base,
                openldap::codes::scopes::LDAP_SCOPE_SUBTREE,
                ldap_filter.as_deref(),
                None,
                false,
                None,
                None,
                ptr::null_mut(),
                -1,
            )
            .ok()?;

        if responses.is_empty() {
            return None;
        }

        let mut gates: Vec<Gate> = responses
            .into_iter()
            .filter_map(|mut response| response.remove("cn"))
            .flat_map(|groups| groups.into_iter())
            .filter_map(|group| self.gate_mappings.get(&group).cloned())
            .flat_map(|groups| groups.into_iter())
            .collect();

        gates.sort_by(|a, b| a.name.cmp(&b.name));
        gates.dedup_by(|a, b| a.name == b.name);

        Some(gates)
    }
}

#[cfg(test)]
pub struct FakeAuth {
    users: HashMap<String, Vec<Gate>>,
}

#[cfg(test)]
impl FakeAuth {
    pub fn new() -> Self {
        FakeAuth {
            users: HashMap::new(),
        }
    }

    pub fn add_user(&mut self, login: &str, password: &str, available_rooms: &[Gate]) {
        self.users
            .insert(format!("{}-{}", login, password), available_rooms.to_vec());
    }
}

#[cfg(test)]
impl Auth for FakeAuth {
    fn get_available_rooms(&self, username: &str, password: &str) -> Option<Vec<Gate>> {
        let key = format!("{}-{}", username, password);

        self.users.get(&key).map(|rooms| rooms.to_owned())
    }
}
