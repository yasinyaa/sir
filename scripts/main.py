import os

from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.asymmetric import padding, rsa
from cryptography.hazmat.primitives.ciphers.aead import AESGCM

# ---------------- Configuration ----------------

RSA_KEY_SIZE = 2048  # bits
RSA_CIPHERTEXT_SIZE = 256  # bytes (2048 / 8)
PAYLOAD_SIZE = 1024 - RSA_CIPHERTEXT_SIZE  # 768 bytes
AES_KEY_SIZE = 32  # 256-bit AES
NONCE_SIZE = 12  # AES-GCM standard

# ---------------- Key Generation ----------------


def generate_rsa_keypair():
    private_key = rsa.generate_private_key(
        public_exponent=65537,
        key_size=RSA_KEY_SIZE,
    )
    return private_key, private_key.public_key()


# ---------------- Padding ----------------


def pad_to_fixed_size(data: bytes, size: int) -> bytes:
    if len(data) > size:
        raise ValueError("Payload too large")
    return data + b"\x00" * (size - len(data))


def unpad(data: bytes) -> bytes:
    return data.rstrip(b"\x00")


# ---------------- Encryption ----------------


def encrypt_message(public_key, plaintext: bytes) -> bytes:
    # 1. Generate symmetric key
    sym_key = os.urandom(AES_KEY_SIZE)

    # 2. Encrypt message with AES-GCM
    aes = AESGCM(sym_key)
    nonce = os.urandom(NONCE_SIZE)
    cipher_text = aes.encrypt(nonce, plaintext, None)

    # 3. Build payload (nonce + ciphertext)
    payload = nonce + cipher_text
    payload = pad_to_fixed_size(payload, PAYLOAD_SIZE)

    # 4. Encrypt symmetric key with RSA
    encrypted_key = public_key.encrypt(
        sym_key,
        padding.OAEP(
            mgf=padding.MGF1(algorithm=hashes.SHA256()),
            algorithm=hashes.SHA256(),
            label=None,
        ),
    )

    # 5. Assemble final message
    final = encrypted_key + payload

    assert len(final) == 1024
    return final


# ---------------- Decryption ----------------


def decrypt_message(private_key, ciphertext: bytes) -> bytes:
    if len(ciphertext) != 1024:
        raise ValueError("Invalid message size")

    # 1. Split message
    encrypted_key = ciphertext[:RSA_CIPHERTEXT_SIZE]
    payload = ciphertext[RSA_CIPHERTEXT_SIZE:]

    # 2. Decrypt symmetric key
    sym_key = private_key.decrypt(
        encrypted_key,
        padding.OAEP(
            mgf=padding.MGF1(algorithm=hashes.SHA256()),
            algorithm=hashes.SHA256(),
            label=None,
        ),
    )

    # 3. Unpad payload
    payload = unpad(payload)

    # 4. Extract nonce + ciphertext
    nonce = payload[:NONCE_SIZE]
    cipher_text = payload[NONCE_SIZE:]

    # 5. Decrypt
    aes = AESGCM(sym_key)
    return aes.decrypt(nonce, cipher_text, None)


# ---------------- Demo ----------------


def main():
    private_key, public_key = generate_rsa_keypair()

    message = b"Hello Alice"
    print("Original:", message)

    encrypted = encrypt_message(public_key, message)
    print("Encrypted length:", encrypted)

    decrypted = decrypt_message(private_key, encrypted)
    print("Decrypted:", decrypted)


if __name__ == "__main__":
    main()
