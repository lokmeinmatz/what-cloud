/**
 * @param {number} mb
 * @returns {string}
 */
export function mbToFormattedString(mb) {
    if (mb < 1000) return `${mb}MB`
    if (mb < 1000000) return `${(mb/1024).toFixed(1)}GB`
    return `${(mb/(1024*1024).toFixed(1))}TB`
}