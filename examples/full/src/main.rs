fn main() {
    use axum_restful::utils::GenerateCertKey;
    struct GenerateAppCertKey;
    impl GenerateCertKey for GenerateAppCertKey {};

    GenerateAppCertKey::generate_cert_key().unwrap();
}
