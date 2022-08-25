import { wordlist } from "./wordlist.js";

function arrayBufferToBase64(arrayBuffer) {
    var byteArray = new Uint8Array(arrayBuffer);
    var byteString = '';
    for(var i=0; i < byteArray.byteLength; i++) {
        byteString += String.fromCharCode(byteArray[i]);
    }
    var b64 = window.btoa(byteString);

    return b64;
}

export function hasKeyPairStored(username){
    return (null != localStorage.getItem('hexstody_key_' + username)) 
}

export function removeStoredKeyPair(username){
    localStorage.removeItem('hexstody_key_' + username)
}

export function listUsers(){
    let keys = []
    for ( var i = 0; i < localStorage.length; ++i ) {
        const key = localStorage.key(i);
        if (key.startsWith("hexstody_key_")){
            keys.push(key.replace("hexstody_key_",""))
        }
    }
    return keys
}

export async function storeKeyPair(username, password, keyPair){
    const pubDer = await keyPair.publicKey.export('der', {encryptParams: {passphrase: password}});
    const encPubDer = Base64.fromUint8Array(pubDer)
    const privDer = await keyPair.privateKey.export('der', {encryptParams: {passphrase: password}})
    const envPrivDer = Base64.fromUint8Array(privDer)
    const storeObj = {priv: envPrivDer, pub: encPubDer}
    localStorage.setItem('hexstody_key_' + username, JSON.stringify(storeObj))
}

export async function retrievePrivateKey(username, password){
    const storedStr = localStorage.getItem("hexstody_key_"+username)
    const storedObj = JSON.parse(storedStr)
    const privBytes = Uint8Array.from(atob(storedObj.priv), c => c.charCodeAt(0));
    const pubBytes = Uint8Array.from(atob(storedObj.pub), c => c.charCodeAt(0));
    let privKey = new window.jscu.Key('der', privBytes);
    let pubKey = new window.jscu.Key('der', pubBytes);
    if (privKey.isEncrypted){
        await privKey.decrypt(password)
    }
    if (pubKey.isEncrypted){
        await pubKey.decrypt(password)
    }
    const keyPair = {publicKey: pubKey, privateKey: privKey}
    return keyPair
}

function toUint11Array(input) {
    var buffer = 0, numbits = 0;
    var output = [];

    for (var i = 0; i < input.length; i++) {
        // prepend bits to buffer
        buffer |= input[i] << numbits;
        numbits += 8;
        // if there are enough bits, extract 11bit chunk
        if (numbits >= 11) {
            output.push(buffer & 0x7FF);
            // drop chunk from buffer
            buffer = buffer >> 11;
            numbits -= 11;
        }
    }
    // also output leftover bits
    if (numbits != 0)
        output.push(buffer);

    return output;
}

export async function keyToMnemonic(key){
    let oct = await key.oct;
    let bytes = Uint8Array.from(oct.match(/.{1,2}/g).map((byte) => parseInt(byte, 16)));
    const hashBuffer = await crypto.subtle.digest('SHA-256', bytes);
    const hashArray = new Uint8Array(hashBuffer)
    const seed = new Uint8Array(33);
    seed.set(bytes);
    seed.set(hashArray.slice(0,1), bytes.length)
    const uint11 = toUint11Array(seed);
    var mnemonic = [];
    for (var i = 0; i < uint11.length; i++) {
        mnemonic.push(wordlist[uint11[i]])
    }
    return mnemonic
}

export async function generateKeyPair(){
    return await jscu.pkc.generateKey('EC', {namedCurve: 'P-256'})
}

async function init(){
    const cat = localStorage.getItem('hexstody_key');
    const yourPassphrase = "qweqwe"
    // console.log(cat)
    
    if (false){
        const key = new window.jscu.Key('pem', cat);
        console.log(key)
        if (key.isEncrypted){
            console.log("enc")
            await key.decrypt(yourPassphrase);
            console.log(key.isEncrypted)
        } else {
            console.log("??")
        }
    } else{

        // case of RSA
        jscu.pkc.generateKey(  // key generation
        'EC', // ECDSA or ECDH key pair
        {namedCurve: 'P-256'} // or 'P-384', 'P-521', 'P-256K'
        )
        .then( async (keyPair) => {
            let key = keyPair.privateKey;
            const mnemonic = await keyToMnemonic(key)
            // console.log(keyPair)
        });
    }
}

document.addEventListener("headerLoaded", init);