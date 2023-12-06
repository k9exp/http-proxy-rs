use std::sync::Arc;
use futures::StreamExt;
use http_mitm_proxy::MitmProxy;

fn make_root_cert() -> rcgen::Certificate {
    let mut param = rcgen::CertificateParams::default();

    param.distinguished_name = rcgen::DistinguishedName::new();
    param.distinguished_name.push(
        rcgen::DnType::CommonName,
        rcgen::DnValue::Utf8String("<HTTP-MITM-PROXY CA>".to_string()),
    );
    param.key_usages = vec![
        rcgen::KeyUsagePurpose::KeyCertSign,
        rcgen::KeyUsagePurpose::CrlSign,
    ];
    param.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    rcgen::Certificate::from_params(param).unwrap()
}


#[tokio::main]
async fn main() {
    let root_cert = Arc::new(make_root_cert());

    let proxy = MitmProxy::new(
        Some(root_cert.clone()), 
        tokio_native_tls::native_tls::TlsConnector::new().unwrap()
    );

    let (mut communications, server) = proxy.bind(("127.0.0.1", 3003)).await.unwrap();
    tokio::spawn(server);

    println!("HTTP Proxy is listening on http://127.0.0.1:3003");

    println!();
    println!("Trust this cert if you want to use HTTPS");
    println!();
    println!("{}", root_cert.serialize_pem().unwrap());
    println!();

    println!(r#"
        Save this cert to ca.crt and use it with curl like this:
        curl https://www.google.com -x https://127.0.0.1:3003 --cacert ca.crt
    "#);

        while let Some(comm) = communications.next().await {
        if let Ok(mut response) = comm.response.await {
            println!(
                "{}\t{}\t{}\t{}",
                comm.client_addr,
                comm.request.uri(),
                response.status(),
                response.body_mut().concat().await.len()
            );
        }
    }
}
