/**
 * @param {number} mb
 * @returns {string}
 */
export function mbToFormattedString(mb) {
    if (mb < 1000) return `${mb}MB`
    if (mb < 1000000) return `${(mb/1024).toFixed(1)}GB`
    return `${(mb/(1024*1024).toFixed(1))}TB`
}

/**
 * @param {number} byte
 * @returns {string}
 */
export function ByteToFormattedString(byte) {
    if (byte < 2**10) return `${byte} B`
    if (byte < 2**20) return `${(byte / (2**10)).toFixed(1)} KiB`
    if (byte < 2**30) return `${(byte / (2**20)).toFixed(1)} MiB`
    if (byte < 2**40) return `${(byte / (2**30)).toFixed(1)} GiB`
    return `${(byte / (2**40)).toFixed(1)} TB`
}

export function randString(length) {
    var result           = '';
    var characters       = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    var charactersLength = characters.length;
    for ( var i = 0; i < length; i++ ) {
       result += characters.charAt(Math.floor(Math.random() * charactersLength));
    }
    return result;
 }