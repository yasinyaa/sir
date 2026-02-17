use crate::utils::crypto::generate_challenge;
use crate::utils::redis::RedisService;
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use actix_web::{HttpResponse, Result, web};
use base64::{Engine as _, engine::general_purpose};
use hex;
use rsa::RsaPublicKey;
use rsa::pkcs1::DecodeRsaPublicKey;
use rsa::pkcs1v15::{Signature, VerifyingKey};
use rsa::pkcs8::DecodePublicKey;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use signature::Verifier;

#[derive(Deserialize)]
pub struct AuthRequest {
    public_key: String,
}

#[derive(Serialize)]
pub struct ChallengeResponse {
    challenge: String,
    expires_at: u64,
}

#[derive(Deserialize)]
pub struct VerifyRequest {
    public_key: String,
    challenge: String,
    signature: String,
}

#[derive(Serialize)]
pub struct VerifiedUserResponse {
    authenticated: bool,
}

pub async fn sign_in(
    body: web::Json<AuthRequest>,
    redis: web::Data<RedisService>,
) -> Result<HttpResponse> {
    let normalized_publickey: Vec<u8> = general_purpose::STANDARD
        .decode(&body.public_key)
        .map_err(|_| ErrorBadRequest("public_key must be base64"))?;

    let digest = Sha256::digest(normalized_publickey);
    let fingerprint = hex::encode(digest);

    let (challenge, expires_at) = generate_challenge();
    let key = format!("challenge:{}", fingerprint);

    redis
        .save_challenge(&challenge, &key, expires_at)
        .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(ChallengeResponse {
        challenge,
        expires_at,
    }))
}

pub async fn verify(
    body: web::Json<VerifyRequest>,
    redis: web::Data<RedisService>,
) -> Result<HttpResponse> {
    let normalized_publickey: Vec<u8> = general_purpose::STANDARD
        .decode(&body.public_key)
        .map_err(|_| ErrorBadRequest("public_key must be base64"))?;

    let digest = Sha256::digest(&normalized_publickey);
    let fingerprint = hex::encode(digest);
    let key = format!("challenge:{}", fingerprint);

    if let Some(stored_challenge) = redis
        .get_challenge(&key)
        .map_err(ErrorInternalServerError)?
    {
        let public_key = RsaPublicKey::from_public_key_der(&normalized_publickey)
            .or_else(|_| RsaPublicKey::from_pkcs1_der(&normalized_publickey))
            .map_err(|_| ErrorBadRequest("invalid RSA public_key DER"))?;

        let signature_bytes = general_purpose::STANDARD
            .decode(&body.signature)
            .map_err(|_| ErrorBadRequest("signature must be base64"))?;
        let signature = Signature::try_from(signature_bytes.as_slice())
            .map_err(|_| ErrorBadRequest("invalid PKCS#1 v1.5 signature bytes"))?;

        let verifying_key = VerifyingKey::<Sha256>::new(public_key);
        let verified = body.challenge == stored_challenge
            && verifying_key
                .verify(body.challenge.as_bytes(), &signature)
                .is_ok();

        redis
            .delete_challenge(&key)
            .map_err(ErrorInternalServerError)?;
        Ok(HttpResponse::Ok().json(VerifiedUserResponse {
            authenticated: verified,
        }))
    } else {
        Ok(HttpResponse::Ok().json(VerifiedUserResponse {
            authenticated: false,
        }))
    }
}
