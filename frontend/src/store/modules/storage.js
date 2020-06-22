import { CachedFS } from '../../business/fs'

export default {
    namespaced: true,
    state: {
        cachedFS: new CachedFS() 
    },
    getters: {
        /**
         * @returns {number} storage used in MB
         */
        storageUsed() {
            return 21312
        }
    }
}