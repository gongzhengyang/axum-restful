use std::{fs, net::SocketAddr, path::PathBuf};

use async_trait::async_trait;
use axum::{
    extract::Host,
    handler::HandlerWithoutStateExt,
    http::{StatusCode, Uri},
    response::Redirect,
};
use axum_server::tls_rustls::RustlsConfig;
use rcgen::{date_time_ymd, Certificate, CertificateParams, DistinguishedName, DnType, SanType};

/// enable https
/// you can config the cert, private key filepath
/// config generate if target filepath is not exists
/// ```rust,no_run
/// use std::net::SocketAddr;
/// use axum::{Router, routing::get};
/// use axum_restful::utils::{GenerateCertKey, redirect_http_to_https};
///
/// struct GenerateAppCertKey;
/// impl GenerateCertKey for GenerateAppCertKey {}
///
/// // config http,https ports
/// let http_port = 3000;
/// let https_port = 3001;
/// let ip = "0.0.0.0";
///
/// // spawn a http service to redirect request to https service
/// # tokio::spawn(async move {
/// redirect_http_to_https(http_port, https_port, ip).await;
/// # });
///
/// let app: Router = Router::new().route("/hello", get(|| async { "Hello, world!" }));
/// # async {
/// let tls_config = GenerateAppCertKey::get_rustls_config(true).await.unwrap();
/// let addr: SocketAddr = format!("{}:{}", ip, https_port).as_str().parse().unwrap();
/// axum_server::bind_rustls(addr, tls_config)
///      .serve(app.into_make_service())
///      .await
///      .unwrap();
/// # };
/// ```
#[async_trait]
pub trait GenerateCertKey {
    fn get_cert_key_path() -> anyhow::Result<(String, String)> {
        fs::create_dir_all("certs/")?;
        Ok(("certs/cert.pem".to_owned(), "certs/key.pem".to_owned()))
    }

    async fn get_rustls_config(create_if_not_exists: bool) -> anyhow::Result<RustlsConfig> {
        let (cert, key) = Self::get_cert_key_path()?;
        let cert_pathbuf = PathBuf::from(cert);
        let key_pathbuf = PathBuf::from(key);
        if create_if_not_exists && (!cert_pathbuf.exists() | !key_pathbuf.exists()) {
            tracing::info!(
                "generate cert at {} and key at {}",
                cert_pathbuf.to_str().unwrap(),
                key_pathbuf.to_str().unwrap()
            );
            Self::generate_cert_key()?;
        }
        Ok(RustlsConfig::from_pem_file(cert_pathbuf, key_pathbuf)
            .await
            .unwrap())
    }

    fn generate_cert_key() -> anyhow::Result<()> {
        let cert = Certificate::from_params(Self::get_cert_params())?;
        let pem_serialized = cert.serialize_pem()?;
        println!("{}", pem_serialized);
        println!("{}", cert.serialize_private_key_pem());
        let (cert_path, key_path) = Self::get_cert_key_path()?;
        fs::write(cert_path, pem_serialized.as_bytes())?;
        fs::write(key_path, cert.serialize_private_key_pem().as_bytes())?;
        Ok(())
    }

    fn get_cert_params() -> CertificateParams {
        let mut params: CertificateParams = Default::default();
        params.not_before = date_time_ymd(1975, 1, 1);
        params.not_after = date_time_ymd(4096, 1, 1);
        params.distinguished_name = DistinguishedName::new();
        params
            .distinguished_name
            .push(DnType::OrganizationName, "Axum-restful");
        params
            .distinguished_name
            .push(DnType::CommonName, "Axum-restful common name");
        params.subject_alt_names = vec![SanType::DnsName("localhost".to_string())];
        params
    }
}

pub async fn redirect_http_to_https(http_port: u16, https_port: u16, http_ip: &str) {
    fn make_https(host: String, uri: Uri, http_port: u16, https_port: u16) -> anyhow::Result<Uri> {
        let mut parts = uri.into_parts();

        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().unwrap());
        }

        let https_host = host.replace(&http_port.to_string(), &https_port.to_string());
        parts.authority = Some(https_host.parse()?);

        Ok(Uri::from_parts(parts)?)
    }

    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(host, uri, http_port, https_port) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(error) => {
                tracing::warn!(%error, "failed to convert URI to HTTPS");
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    let addr: SocketAddr = format!("{}:{}", http_ip, http_port)
        .as_str()
        .parse()
        .unwrap();
    tracing::debug!("http redirect listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(redirect.into_make_service())
        .await
        .unwrap();
}
