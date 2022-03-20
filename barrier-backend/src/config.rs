use crate::structs::Gate;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, process};

#[derive(Serialize, Deserialize)]
pub struct Ldap {
    pub server: String,
    pub base: String,
    pub bind: String,
    pub filter: Option<String>,
}

fn default_dry_run() -> bool {
    false
}

fn default_log_level() -> String {
    "warn".to_string()
}

fn default_listen_addr() -> String {
    "127.0.0.1:7000".to_string()
}

fn default_retries() -> i32 {
    1
}

#[derive(Serialize, Deserialize)]
pub struct ConfigGate {
    pub id: i32,
    pub description: String,
    #[serde(default = "default_retries")]
    pub retries: i32,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,

    pub jwt_key: String,

    pub mongo_uri: String,

    #[serde(default = "default_dry_run")]
    pub dry_run: bool,

    #[serde(default = "default_log_level")]
    pub log_level: String,

    pub gate_server: String,
    pub gates: HashMap<String, Vec<String>>,
    pub gate_mapping: HashMap<String, ConfigGate>,
    pub ldap: Ldap,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:7000".to_string(),
            jwt_key: "PLEASE FILL JWT KEY".to_string(),
            mongo_uri: "mongo://please-fill-uri/".to_string(),
            dry_run: false,
            log_level: "warn".to_string(),
            gate_server: "PLEASE FILL GATE SERVER ADDRESS".to_string(),
            gates: {
                let mut example = HashMap::new();
                example.insert("group".to_string(), vec!["Gate".to_string()]);
                example
            },
            gate_mapping: {
                let mut example = HashMap::new();
                example.insert(
                    "Gate".to_string(),
                    ConfigGate {
                        id: 1,
                        description: "Example gate".to_string(),
                        retries: 1,
                    },
                );
                example
            },
            ldap: Ldap {
                server: "PLEASE FILL LDAP SERVER ADDRESS".to_string(),
                base: "PLEASE FILL LDAP BASE".to_string(),
                bind: "PLEASE FILL LDAP BIND".to_string(),
                filter: Some("PLEASE FILL LDAP FILTER OR DELETE THIS LINE".to_string()),
            },
        }
    }
}

impl Config {
    pub fn new() -> Self {
        if let Some(config_file) = std::env::args().nth(1) {
            let s = fs::read_to_string(&config_file).expect("config.toml");

            return toml::from_str(&s).expect("true toml file");
        }

        let mut cfg_dir = dirs::config_dir().expect("Config directory (like :/home/user/.config)");

        cfg_dir.push("barrier");

        let mut cfg_file = cfg_dir.clone();
        cfg_file.push("config.toml");

        if !cfg_dir.is_dir() {
            fs::create_dir_all(&cfg_dir).expect("create config directory");
        }

        if !cfg_file.is_file() {
            let config = Config::default();
            fs::write(
                &cfg_file,
                toml::to_string(&config).expect("config to string"),
            )
            .expect("write file");

            eprintln!("Please configure {}", cfg_file.display());
            process::exit(1);
        }

        let s = fs::read_to_string(&cfg_file).expect("config.toml");

        toml::from_str(&s).expect("true toml file")
    }

    pub fn get_mappings(&self) -> HashMap<String, Vec<Gate>> {
        self.gates
            .clone()
            .into_iter()
            .map(|(group, gates)| {
                (
                    group,
                    gates
                        .into_iter()
                        .filter_map(|gate_name| {
                            self.gate_mapping.get(&gate_name).map(|gate| Gate {
                                id: gate.id,
                                name: gate_name,
                                description: gate.description.clone(),
                                retries: gate.retries,
                            })
                        })
                        .collect(),
                )
            })
            .collect()
    }
}
