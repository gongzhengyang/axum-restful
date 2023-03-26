use std::{fs, net::SocketAddr, path::PathBuf};

use async_trait::async_trait;
use axum::{
    extract::Host,
    handler::HandlerWithoutStateExt,
    http::{StatusCode, Uri},
    response::Redirect,
    BoxError, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use rcgen::{date_time_ymd, Certificate, CertificateParams, DistinguishedName, DnType, SanType};

use crate::AppError;

#[derive(Clone, Copy)]
pub struct Ports {
    pub http: u16,
    pub https: u16,
}

#[async_trait]
pub trait GenerateCertKey {
    fn get_cert_key_path() -> (String, String) {
        ("cert/cert.pem".to_owned(), "cert/key.pem".to_owned())
    }

    async fn get_rustls_config(create_if_not_esists: bool) -> RustlsConfig {
        let (cert, key) = Self::get_cert_key_path();
        let cert_pathbuf = PathBuf::from(cert);
        let key_pathbuf = PathBuf::from(key);
        if create_if_not_esists && (!cert_pathbuf.exists() | !key_pathbuf.exists()) {
            tracing::info!(
                "generate cert at {} and key at {}",
                cert_pathbuf.to_str().unwrap(),
                key_pathbuf.to_str().unwrap()
            );
            Self::generate_cert_key();
        }
        RustlsConfig::from_pem_file(cert_pathbuf, key_pathbuf)
            .await
            .unwrap()
    }

    fn generate_cert_key() -> Result<(), AppError> {
        let cert = Certificate::from_params(Self::get_cert_params())?;

        let pem_serialized = cert.serialize_pem()?;
        let der_serialized = pem::parse(&pem_serialized).unwrap().contents;
        println!("{}", pem_serialized);
        println!("{}", cert.serialize_private_key_pem());
        std::fs::create_dir_all("certs/")?;
        fs::write("certs/cert.pem", &pem_serialized.as_bytes())?;
        fs::write("certs/cert.der", &der_serialized)?;
        fs::write(
            "certs/key.pem",
            &cert.serialize_private_key_pem().as_bytes(),
        )?;
        fs::write("certs/key.der", &cert.serialize_private_key_der())?;

        Ok(())
    }

    fn get_cert_params() -> CertificateParams {
        let mut params: CertificateParams = Default::default();
        params.not_before = date_time_ymd(1975, 01, 01);
        params.not_after = date_time_ymd(4096, 01, 01);
        params.distinguished_name = DistinguishedName::new();
        params
            .distinguished_name
            .push(DnType::OrganizationName, "Crab widgits SE");
        params
            .distinguished_name
            .push(DnType::CommonName, "Master Cert");
        params.subject_alt_names = vec![
            SanType::DnsName("crabs.crabs".to_string()),
            SanType::DnsName("localhost".to_string()),
        ];
        params
    }
}

// pub async fn tls_server(addr: SocketAddr, app: Router, generate_app_cert_key: Box<dyn GenerateCertKey>) {
//     let config = <generate_app_cert_key as GenerateCertKey>::get_rustls_config(true);
//     axum_server::bind_rustls(addr, config)
//         .serve(app.into_make_service())
//         .await
//         .unwrap();
// }

fn make_https(host: String, uri: Uri, ports: Ports) -> Result<Uri, BoxError> {
    let mut parts = uri.into_parts();

    parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

    if parts.path_and_query.is_none() {
        parts.path_and_query = Some("/".parse().unwrap());
    }

    let https_host = host.replace(&ports.http.to_string(), &ports.https.to_string());
    parts.authority = Some(https_host.parse()?);

    Ok(Uri::from_parts(parts)?)
}

pub async fn redirect_http_to_https(ports: Ports) {
    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(host, uri, ports) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(error) => {
                tracing::warn!(%error, "failed to convert URI to HTTPS");
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], ports.http));
    tracing::debug!("http redirect listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(redirect.into_make_service())
        .await
        .unwrap();
}
