use erased_serde::serialize_trait_object;
use serde::Serialize;
use std::collections::HashMap;

// https://google.aip.dev/auth/4117
#[derive(Serialize)]
pub struct ExternalAccountKey {
    pub r#type: String,
    pub audience: String,
    pub subject_token_type: String,
    pub token_uri: String,
    pub service_account_impersonation_url: String,
    pub credential_source: Box<dyn CredentialSourceTrait>,
    // #[serde(serialize_with = "STSCredentialSource::custom_serialize")]
    pub credential_source_type: STSCredentialSource,
}

fn default_sts_external_account_key() -> ExternalAccountKey {
    ExternalAccountKey {
        r#type: String::from("external_account"),
        audience: String::from("//iam.googleapis.com/projects/1234567890/location/global/workloadIdentityPools/BLAH-pool/providers/BLAH-provider"),
        subject_token_type: String::from("urn:ietf:params:oath:token-type:jwt"),
        token_uri: String::from("https://sts.googleapis.com/v1/token"),
        service_account_impersonation_url: Default::default(),
        credential_source: Box::new(URLSourced {
            url: String::from("https://iam.BLAH.com/auth/v1/get-token?audience=gcp-wip.BLAH.com"),
            headers: HashMap::from([(String::from("X-IAM"), String::from("1"))]),
            format_spec: FormatSpec {
                r#type: String::from("json"),
                subject_token_field_name: String::from("access_token"),
            },
        }),
        credential_source_type: STSCredentialSource::URLSourced,
    }
}

impl Default for ExternalAccountKey {
    fn default() -> ExternalAccountKey {
        default_sts_external_account_key()
    }
}

//What is Debug

// https://google.aip.dev/auth/4117#determining-the-subject-token-in-microsoft-azure-and-url-sourced-credentials
#[derive(Serialize)]
pub struct URLSourced {
    pub url: String,
    pub headers: HashMap<String, String>,
    #[serde(rename = "format")]
    pub format_spec: FormatSpec,
}

#[derive(Serialize)]
pub struct FormatSpec {
    pub r#type: String,
    pub subject_token_field_name: String,
}

// https://google.aip.dev/auth/4117#determining-the-subject-token-in-executable-sourced-credentials
#[derive(Serialize)]
pub struct ExecutableSourced {
    pub executable: Executable,
}

#[derive(Serialize)]
pub struct Executable {
    pub command: String,
    pub timeout_millis: i32,
}

pub trait CredentialSourceTrait: erased_serde::Serialize {}
impl CredentialSourceTrait for URLSourced {}
impl CredentialSourceTrait for ExecutableSourced {}
serialize_trait_object!(CredentialSourceTrait);

pub enum STSCredentialSource {
    URLSourced,
    ExecutableSourced,
}

impl STSCredentialSource {
    fn from_str(s: &str) -> Result<STSCredentialSource, String> {
        match s {
            "URLSourced" => Ok(STSCredentialSource::URLSourced),
            "ExecutableSourced" => Ok(STSCredentialSource::ExecutableSourced),
            _ => Err(format!("Invalid input: {}", s)),
        }
    }
}

impl Serialize for STSCredentialSource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            STSCredentialSource::URLSourced =>
                serde_json::json!({
                    "url": "https://iam.BLAH.com/auth/v1/get-token?audience=gcp-wip.BLAH.com",
                    "headers": {
                        "X-IAM": "1"
                    },
                    "format": {
                        "type": "json",
                        "subject_token_field_name": "access_token"
                    }
                }).serialize(serializer),
            STSCredentialSource::ExecutableSourced =>
                serde_json::json!({
                    "executable": {
                        "command": "/opt/bin/blah_blah_blah.sh https://blah.com/auth/v1/ext/get-token?audience=gcp-wip.BLAH.com",
                        "timeout_millis": 5000
                    }
                }).serialize(serializer),
        }
    }
}

fn main() {
    let service_account = String::from("boo@boo.com");

    let credential_source: Box<dyn CredentialSourceTrait> = if false {
        Box::new(URLSourced {
            url: String::from("https://iam.BLAH.com/auth/v1/get-token?audience=gcp-wip.BLAH.com"),
            headers: HashMap::from([(String::from("X-IAM"), String::from("1"))]),
            format_spec: FormatSpec {
                r#type: String::from("json"),
                subject_token_field_name: String::from("access_token"),
            },
        })
    } else {
        Box::new(ExecutableSourced {
            executable: Executable {
                command: String::from("/opt/bin/blah_blah_blah.sh https://blah.com/auth/v1/ext/get-token?audience=gcp-wip.BLAH.com"),
                timeout_millis: 5000,
            },
        })
    };

    let env_sourced = std::env::var("SOURCED").unwrap_or(String::from("URLSourced"));
    let sts_credential_source = STSCredentialSource::from_str(&env_sourced).unwrap();

    let key = ExternalAccountKey {
        service_account_impersonation_url: String::from(format!("https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/{}:generateAccessToken", service_account)),
        credential_source: credential_source,
        credential_source_type: sts_credential_source,
        ..default_sts_external_account_key()
    };

    let serialized = serde_json::to_string(&key).unwrap();
    println!("{}", serialized)
}
