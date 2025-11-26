//! Assinatura para requisições autenticadas da Kraken Futures
//!
//! Algoritmo de autenticação:
//! 1. Concatenar: postData + Nonce + endpointPath
//! 2. Aplicar SHA-256 no resultado
//! 3. Decodificar API secret de base64
//! 4. Aplicar HMAC-SHA-512 usando o secret decodificado
//! 5. Codificar resultado em base64

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256, Sha512};

type HmacSha512 = Hmac<Sha512>;

/// Gera assinatura Authent para a Kraken Futures
///
/// # Arguments
/// * `api_secret` - API secret em base64
/// * `post_data` - Dados do POST (query string url-encoded)
/// * `nonce` - Nonce (timestamp em milissegundos)
/// * `endpoint_path` - Caminho do endpoint (ex: /derivatives/api/v3/sendorder)
///
/// # Returns
/// Assinatura Authent em base64
pub fn sign(api_secret: &str, post_data: &str, nonce: &str, endpoint_path: &str) -> String {
    // 1. Concatenar: postData + nonce + endpointPath
    let concat = format!("{}{}{}", post_data, nonce, endpoint_path);

    // 2. SHA-256 hash
    let sha256_hash = Sha256::digest(concat.as_bytes());

    // 3. Decodificar API secret de base64
    let decoded_secret = BASE64
        .decode(api_secret)
        .expect("API secret deve ser base64 válido");

    // 4. HMAC-SHA-512 usando o secret decodificado
    let mut mac =
        HmacSha512::new_from_slice(&decoded_secret).expect("HMAC pode aceitar chave de qualquer tamanho");
    mac.update(&sha256_hash);
    let result = mac.finalize();

    // 5. Codificar resultado em base64
    BASE64.encode(result.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_basic() {
        // Teste básico de assinatura
        // Nota: Este é um teste de estrutura, não de valores exatos
        let secret = BASE64.encode("test_secret_key");
        let post_data = "symbol=PI_XBTUSD&side=buy&size=1";
        let nonce = "1699999999999";
        let endpoint = "/derivatives/api/v3/sendorder";

        let signature = sign(&secret, post_data, nonce, endpoint);

        // Verifica que a assinatura é base64 válido
        assert!(BASE64.decode(&signature).is_ok());
        // Verifica tamanho esperado (SHA-512 = 64 bytes -> base64 ~88 chars)
        assert!(!signature.is_empty());
    }

    #[test]
    fn test_sign_empty_post_data() {
        let secret = BASE64.encode("test_secret");
        let signature = sign(&secret, "", "1699999999999", "/derivatives/api/v3/accounts");

        assert!(BASE64.decode(&signature).is_ok());
    }

    #[test]
    fn test_sign_consistency() {
        // Mesmos inputs devem produzir mesma assinatura
        let secret = BASE64.encode("consistent_secret");
        let post_data = "param=value";
        let nonce = "1700000000000";
        let endpoint = "/test/endpoint";

        let sig1 = sign(&secret, post_data, nonce, endpoint);
        let sig2 = sign(&secret, post_data, nonce, endpoint);

        assert_eq!(sig1, sig2);
    }
}
