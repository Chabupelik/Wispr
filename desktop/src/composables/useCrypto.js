// frontend/src/composables/useCrypto.js
export function useCrypto() {
  const enc = new TextEncoder();
  const dec = new TextDecoder();

  function uint8ToBase64(bytes) {
    let binary = '';
    const chunkSize = 0x8000; // 32768 — безопасно ниже лимита аргументов функции
    for (let i = 0; i < bytes.length; i += chunkSize) {
      binary += String.fromCharCode.apply(null, bytes.subarray(i, i + chunkSize));
    }
    return btoa(binary);
  }

  // Генерация 256-битного AES-GCM ключа на основе пароля юзера (KDF)
  const deriveKey = async (password, salt = 'scramjet-salt-static') => {
    const keyMaterial = await crypto.subtle.importKey(
      'raw',
      enc.encode(password),
      { name: 'PBKDF2' },
      false,
      ['deriveBits', 'deriveKey']
    );

    return crypto.subtle.deriveKey(
      {
        name: 'PBKDF2',
        salt: enc.encode(salt),
        iterations: 100000,
        hash: 'SHA-256'
      },
      keyMaterial,
      { name: 'AES-GCM', length: 256 },
      false, // Ключ нельзя извлечь из WebCrypto
      ['encrypt', 'decrypt']
    );
  };

  // Шифрование данных для отправки на сервер
  const encryptData = async (dataObject, aesKey) => {
    const iv = crypto.getRandomValues(new Uint8Array(12));
    const encodedData = enc.encode(JSON.stringify(dataObject));

    const encryptedBuffer = await crypto.subtle.encrypt(
      { name: 'AES-GCM', iv },
      aesKey,
      encodedData
    );

    const encryptedBase64 = uint8ToBase64(new Uint8Array(encryptedBuffer)); // <-- было .apply на весь массив разом
    const ivBase64 = uint8ToBase64(iv);

    return { payload: encryptedBase64, iv: ivBase64 };
  };

  const decryptData = async (encryptedBase64, ivBase64, aesKey) => {
    try {
      const encryptedBytes = Uint8Array.from(atob(encryptedBase64), c => c.charCodeAt(0));
      const ivBytes = Uint8Array.from(atob(ivBase64), c => c.charCodeAt(0));

      const decryptedBuffer = await crypto.subtle.decrypt(
        { name: 'AES-GCM', iv: ivBytes },
        aesKey,
        encryptedBytes
      );

      return JSON.parse(dec.decode(decryptedBuffer));
    } catch (e) {
      console.error('Ошибка расшифровки данных', e);
      return null;
    }
  };

  return { deriveKey, encryptData, decryptData };
}