# CopyPaws Encryption Specification

## Algorithm

- **Cipher:** AES-256-GCM
- **Key Size:** 256 bits (32 bytes)
- **IV Size:** 96 bits (12 bytes)
- **Tag Size:** 128 bits (16 bytes, appended to ciphertext)

---

## Key Exchange

1. Desktop generates random 32-byte `shared_secret`
2. Encoded as Base64 in QR code
3. Mobile scans and stores in Keychain/Keystore
4. Same key used for all messages between this device pair

---

## Encryption Process

### Sending (CLIP_PUSH)

1. Plaintext: UTF-8 encoded string
2. Generate random 12-byte IV
3. Encrypt using AES-256-GCM with `shared_secret`
4. Base64-encode ciphertext and IV
5. Send in message

### Receiving (ENCRYPTED wrapper)

1. Base64-decode `payload` and `iv`
2. Decrypt using AES-256-GCM with stored key
3. Parse decrypted string as JSON
4. Handle inner message based on `type`

---

## Code Examples

### JavaScript (Web Crypto API)

```javascript
async function encrypt(plaintext, key) {
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const encoded = new TextEncoder().encode(plaintext);
  
  const ciphertext = await crypto.subtle.encrypt(
    { name: 'AES-GCM', iv },
    key,
    encoded
  );
  
  return {
    encrypted: btoa(String.fromCharCode(...new Uint8Array(ciphertext))),
    iv: btoa(String.fromCharCode(...iv))
  };
}

async function decrypt(base64Ciphertext, base64Iv, key) {
  const ciphertext = Uint8Array.from(atob(base64Ciphertext), c => c.charCodeAt(0));
  const iv = Uint8Array.from(atob(base64Iv), c => c.charCodeAt(0));
  
  const plaintext = await crypto.subtle.decrypt(
    { name: 'AES-GCM', iv },
    key,
    ciphertext
  );
  
  return new TextDecoder().decode(plaintext);
}
```

### Dart (Flutter)

```dart
import 'package:pointycastle/export.dart';

Uint8List encrypt(String plaintext, Uint8List key) {
  final iv = generateRandomBytes(12);
  final cipher = GCMBlockCipher(AESEngine());
  final params = AEADParameters(KeyParameter(key), 128, iv, Uint8List(0));
  cipher.init(true, params);
  
  final input = utf8.encode(plaintext);
  final output = Uint8List(input.length + 16);
  cipher.processBytes(input, 0, input.length, output, 0);
  cipher.doFinal(output, input.length);
  
  return output;
}
```

### Rust

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<(Vec<u8>, [u8; 12])> {
    let cipher = Aes256Gcm::new(Key::from_slice(key));
    let nonce = generate_random_nonce();
    let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), plaintext)?;
    Ok((ciphertext, nonce))
}
```

---

## Security Notes

1. **IV must be unique** for each message with the same key
2. **Never reuse IV** - generate new random IV each time
3. **Store key securely** - Keychain (iOS), Keystore (Android)
4. **Clear key from memory** when revoking device
