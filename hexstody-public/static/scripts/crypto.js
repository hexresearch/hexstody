import { wordlist } from "./wordlist.js";

const wordlistArray = Object.keys(wordlist)

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

export async function storePrivateKey(username, password, privateKey){
    const privDer = await privateKey.export('der', {encryptParams: {passphrase: password}})
    const encPrivDer = Base64.fromUint8Array(privDer)
    localStorage.setItem('hexstody_key_' + username, JSON.stringify(encPrivDer))
}

export async function retrievePrivateKey(username, password){
    const storedStr = localStorage.getItem("hexstody_key_"+username)
    const encPrivDer = JSON.parse(storedStr)
    const privBytes = Uint8Array.from(atob(encPrivDer), c => c.charCodeAt(0));
    let privKey = new window.jscu.Key('der', privBytes);
    if (privKey.isEncrypted){
        await privKey.decrypt(password)
    }
    return privKey
}

// Transform decimal to bitwise representation and fill to size with leading zeroes
function dec2binFillN(dec, n){
    const str = dec.toString(2);
    if (str.length < n){
        return "0".repeat(n - str.length) + str
    } else {
        return str
    }
}

function toUint11Array(input){
    let inpStr = "";
    for (let i=0; i<input.length; i++){
        inpStr += dec2binFillN(input[i],8)
    }
    return inpStr.match(/.{1,11}/g).map(b => parseInt(b,2));
}

function fromUint11Array(input){
    let inpStr = "";
    for (let i=0; i<input.length; i++){
        inpStr += dec2binFillN(input[i],11)
    }
    return new Uint8Array(inpStr.match(/.{1,8}/g).map(b => parseInt(b,2)))
}

export async function privateKeyToMnemonic(key){
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
        mnemonic.push(wordlistArray[uint11[i]])
    }
    return mnemonic
}

export async function mnemonicToPrivateKey(mnemonicArray){
    if (mnemonicArray.length != 24) {
        return {ok: false, error: "Not enough words!"}
    }
    var mnemBytes = []
    for (var i = 0; i < mnemonicArray.length; i++) {
        const byte = wordlist[mnemonicArray[i]]
        if(byte){
            mnemBytes.push(byte)
        } else{
            return {ok: false, error: mnemonicArray[i] + " is not a valid mnemonic word"}
        }
    }
    const bytes = fromUint11Array(mnemBytes)
    const keyBytes = bytes.slice(0, 32);
    const checkSum = bytes[32];
    const hashBuffer = await crypto.subtle.digest('SHA-256', keyBytes);
    const hashByte = new Uint8Array(hashBuffer)[0];
    if (hashByte != checkSum){
        return {ok: false, error: "Checksum failure"}
    }
    const priv_key = new window.jscu.Key('oct', keyBytes, {namedCurve: 'P-256'});
    return {ok: true, value: priv_key}
}

export async function generateKeyPair(){
    return await jscu.pkc.generateKey('EC', {namedCurve: 'P-256'})
}

async function init(){
    const keyPair = await generateKeyPair();
}

document.addEventListener("headerLoaded", init);