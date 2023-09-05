use openssl::{pkey::PKey, rsa::Rsa};
use std::io::{Error, ErrorKind};

pub struct Keypair {
    pub private_key: String,
    pub public_key: String,
}

/// Generate the asymmetric keypair for ActivityPub HTTP signatures.
pub fn generate_actor_keypair() -> Result<Keypair, Error> {
    let rsa = Rsa::generate(2048)?;
    let pkey = PKey::from_rsa(rsa)?;
    let public_key = pkey.public_key_to_pem()?;
    let private_key = pkey.private_key_to_pem_pkcs8()?;
    let key_to_string = |key| match String::from_utf8(key) {
        Ok(s) => Ok(s),
        Err(e) => Err(Error::new(
            ErrorKind::Other,
            format!("Failed converting key to string: {e}"),
        )),
    };
    Ok(Keypair {
        private_key: key_to_string(private_key)?,
        public_key: key_to_string(public_key)?,
    })
}
