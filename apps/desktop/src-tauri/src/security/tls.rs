use crate::errors::{IMirrorError, IMirrorResult};
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DistinguishedName, DnType,
    ExtendedKeyUsagePurpose, IsCa, KeyUsagePurpose, SanType, PKCS_ECDSA_P256_SHA256,
};
use std::{fs, net::IpAddr, path::PathBuf};

const CERT_FILE: &str = "imirror-local-ca.pem";
const CERT_DER_FILE: &str = "imirror-local-ca.cer";
const KEY_FILE: &str = "imirror-local-ca-key.pem";
const HOST_FILE: &str = "host.txt";

#[derive(Clone)]
pub struct LocalTlsIdentity {
    pub cert_pem: Vec<u8>,
    pub key_pem: Vec<u8>,
    pub cert_der: Vec<u8>,
    pub host: String,
}

pub fn load_or_create(host: &str) -> IMirrorResult<LocalTlsIdentity> {
    let directory = certificate_directory()?;
    fs::create_dir_all(&directory).map_err(|error| tls_error("create TLS directory", error))?;

    let cert_path = directory.join(CERT_FILE);
    let cert_der_path = directory.join(CERT_DER_FILE);
    let key_path = directory.join(KEY_FILE);
    let host_path = directory.join(HOST_FILE);

    let stored_host = fs::read_to_string(&host_path).unwrap_or_default();
    if stored_host.trim() == host
        && cert_path.is_file()
        && cert_der_path.is_file()
        && key_path.is_file()
    {
        return Ok(LocalTlsIdentity {
            cert_pem: fs::read(cert_path)
                .map_err(|error| tls_error("read TLS certificate", error))?,
            key_pem: fs::read(key_path)
                .map_err(|error| tls_error("read TLS private key", error))?,
            cert_der: fs::read(cert_der_path)
                .map_err(|error| tls_error("read iPhone certificate", error))?,
            host: host.to_string(),
        });
    }

    let mut params = CertificateParams::new(vec!["localhost".to_string()]);
    if let Ok(ip) = host.parse::<IpAddr>() {
        params.subject_alt_names.push(SanType::IpAddress(ip));
    } else {
        params
            .subject_alt_names
            .push(SanType::DnsName(host.to_string()));
    }
    params.alg = &PKCS_ECDSA_P256_SHA256;
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyEncipherment,
        KeyUsagePurpose::KeyCertSign,
    ];
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    let mut distinguished_name = DistinguishedName::new();
    distinguished_name.push(DnType::CommonName, "iMirror Local Camera");
    distinguished_name.push(DnType::OrganizationName, "iMirror");
    params.distinguished_name = distinguished_name;

    let certificate = Certificate::from_params(params)
        .map_err(|error| tls_error("generate local TLS certificate", error))?;
    let cert_pem = certificate
        .serialize_pem()
        .map_err(|error| tls_error("encode local TLS certificate", error))?
        .into_bytes();
    let key_pem = certificate.serialize_private_key_pem().into_bytes();
    let cert_der = certificate
        .serialize_der()
        .map_err(|error| tls_error("encode iPhone certificate", error))?;

    fs::write(&cert_path, &cert_pem).map_err(|error| tls_error("save TLS certificate", error))?;
    fs::write(&cert_der_path, &cert_der)
        .map_err(|error| tls_error("save iPhone certificate", error))?;
    fs::write(&key_path, &key_pem).map_err(|error| tls_error("save TLS private key", error))?;
    fs::write(&host_path, host).map_err(|error| tls_error("save TLS host", error))?;

    Ok(LocalTlsIdentity {
        cert_pem,
        key_pem,
        cert_der,
        host: host.to_string(),
    })
}

fn certificate_directory() -> IMirrorResult<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .map(|path| path.join("iMirror").join("tls"))
        .ok_or_else(|| {
            IMirrorError::new(
                "tls-storage-unavailable",
                "Windows local application data directory is unavailable.",
            )
        })
}

fn tls_error(context: &str, error: impl std::fmt::Display) -> IMirrorError {
    IMirrorError::new("tls-setup-failed", format!("Could not {context}."))
        .with_detail(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn certificate_params_accept_lan_ip() {
        let mut params = CertificateParams::new(vec!["localhost".to_string()]);
        params
            .subject_alt_names
            .push(SanType::IpAddress("192.168.1.10".parse().unwrap()));
        assert!(params.subject_alt_names.len() >= 2);
    }
}
