const SALT = import.meta.env.VITE_WISP_SALT;

async function getEncryptionKey() {
    const enc = new TextEncoder();
    // Actually we just use SHA-256 of the salt as the key to match Rust and Node
    const hash = await window.crypto.subtle.digest("SHA-256", enc.encode(SALT));
    
    return await window.crypto.subtle.importKey(
        "raw",
        hash,
        { name: "AES-GCM" },
        false,
        ["encrypt", "decrypt"]
    );
}

export async function encryptPayload(dataObj) {
    const key = await getEncryptionKey();
    const iv = window.crypto.getRandomValues(new Uint8Array(12));
    const enc = new TextEncoder();
    
    const ciphertextBuf = await window.crypto.subtle.encrypt(
        { name: "AES-GCM", iv: iv },
        key,
        enc.encode(JSON.stringify(dataObj))
    );
    
    // ciphertextBuf includes the Auth Tag at the end (16 bytes)
    const payload = new Uint8Array(12 + ciphertextBuf.byteLength);
    payload.set(iv, 0);
    payload.set(new Uint8Array(ciphertextBuf), 12);
    
    return payload; // Uint8Array
}

export async function decryptPayload(payloadBuffer) {
    const key = await getEncryptionKey();
    const payload = new Uint8Array(payloadBuffer);
    
    const iv = payload.slice(0, 12);
    const ciphertextAndTag = payload.slice(12);
    
    const decryptedBuf = await window.crypto.subtle.decrypt(
        { name: "AES-GCM", iv: iv },
        key,
        ciphertextAndTag
    );
    
    const dec = new TextDecoder();
    return JSON.parse(dec.decode(decryptedBuf));
}
