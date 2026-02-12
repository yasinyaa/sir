import asyncio
import os
from pathlib import Path

import websockets
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import padding, rsa
from cryptography.hazmat.primitives.ciphers.aead import AESGCM

# ---------------- Configuration ----------------

WS_URL = "ws://127.0.0.1:8080/ws"

RSA_KEY_SIZE = 2048
RSA_CIPHERTEXT_SIZE = 256
TOTAL_SIZE = 1024
PAYLOAD_SIZE = TOTAL_SIZE - RSA_CIPHERTEXT_SIZE

AES_KEY_SIZE = 32
NONCE_SIZE = 12

PRIAVTE_KEY_PATH = Path("private.pem")
PUBLIC_KEY_PATH = Path("public.pem")

# ---------------- Key Generation ----------------


def generate_rsa_keypair():
    private_key = rsa.generate_private_key(
        public_exponent=65537,
        key_size=RSA_KEY_SIZE,
    )
    public_key = private_key.public_key()
    private_pem = private_key.private_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PrivateFormat.PKCS8,
        encryption_algorithm=serialization.NoEncryption(),
    )

    public_pem = public_key.public_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PublicFormat.SubjectPublicKeyInfo,
    )

    with open(PRIAVTE_KEY_PATH, "wb") as f:
        f.write(private_pem)

    with open(PUBLIC_KEY_PATH, "wb") as f:
        f.write(public_pem)

    return


def load_rsa_keypair():
    with open("private.pem", "rb") as f:
        private_key = serialization.load_pem_private_key(f.read(), password=None)

    with open("public.pem", "rb") as f:
        public_key = serialization.load_pem_public_key(f.read())

    return private_key, public_key


def keys_generated():
    print(PRIAVTE_KEY_PATH.is_file() and PUBLIC_KEY_PATH.is_file())
    if PRIAVTE_KEY_PATH.is_file() and PUBLIC_KEY_PATH.is_file():
        return True
    return False


# ---------------- Padding ----------------


def pad(data: bytes, size: int) -> bytes:
    if len(data) > size:
        raise ValueError("Payload too large")
    return data + b"\x00" * (size - len(data))


def unpad(data: bytes) -> bytes:
    return data.rstrip(b"\x00")


# ---------------- Encryption ----------------


def encrypt_message(public_key, plaintext: bytes) -> bytes:
    sym_key = os.urandom(AES_KEY_SIZE)

    aes = AESGCM(sym_key)
    nonce = os.urandom(NONCE_SIZE)
    cipher_text = aes.encrypt(nonce, plaintext, None)

    payload = pad(nonce + cipher_text, PAYLOAD_SIZE)

    encrypted_key = public_key.encrypt(
        sym_key,
        padding.OAEP(
            mgf=padding.MGF1(algorithm=hashes.SHA256()),
            algorithm=hashes.SHA256(),
            label=None,
        ),
    )

    final = encrypted_key + payload
    assert len(final) == TOTAL_SIZE
    return final


# ---------------- Decryption ----------------


def decrypt_message(private_key, ciphertext: bytes) -> bytes:
    if len(ciphertext) != TOTAL_SIZE:
        raise ValueError("Invalid ciphertext size")

    encrypted_key = ciphertext[:RSA_CIPHERTEXT_SIZE]
    payload = ciphertext[RSA_CIPHERTEXT_SIZE:]

    sym_key = private_key.decrypt(
        encrypted_key,
        padding.OAEP(
            mgf=padding.MGF1(algorithm=hashes.SHA256()),
            algorithm=hashes.SHA256(),
            label=None,
        ),
    )

    payload = unpad(payload)
    nonce = payload[:NONCE_SIZE]
    cipher_text = payload[NONCE_SIZE:]

    aes = AESGCM(sym_key)
    return aes.decrypt(nonce, cipher_text, None)


# ---------------- WebSocket Tasks ----------------


async def receiver(ws, private_key):
    """Receive, decrypt, and print messages."""
    while True:
        msg = await ws.recv()

        if not isinstance(msg, bytes):
            print("⚠ Received non-binary frame")
            continue

        try:
            plaintext = decrypt_message(private_key, msg)
            print(f"\n⬅ Received (decrypted): {plaintext.decode(errors='ignore')}")
        except Exception as e:
            print("\n❌ Failed to decrypt message:", e)


async def sender(ws, public_key):
    """Read user input, encrypt, and send."""
    loop = asyncio.get_event_loop()

    while True:
        text = await loop.run_in_executor(None, input, "➡ Enter message: ")
        plaintext = text.encode()

        encrypted = encrypt_message(public_key, plaintext)
        await ws.send(encrypted)

        print(f"✔ Sent {len(encrypted)} bytes")


# ---------------- Main ----------------


async def ws_client():
    if not keys_generated():
        generate_rsa_keypair()

    private_key, public_key = load_rsa_keypair()

    async with websockets.connect(WS_URL, max_size=None) as ws:
        print("🔐 Connected to encrypted WebSocket")

        await asyncio.gather(
            receiver(ws, private_key),
            sender(ws, public_key),
        )


if __name__ == "__main__":
    try:
        asyncio.run(ws_client())
    except Exception as e:
        print(e)
