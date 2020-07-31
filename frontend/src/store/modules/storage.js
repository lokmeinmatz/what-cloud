import { ActionContext } from 'vuex'


/**
 * @field {} name
 */
class Folder {
    constructor(name, childrenFolder, files, pathFromRoot) {
        this.name = name
        /**
         * @type {Folder[]}
         */
        this.childrenFolder = childrenFolder
        this.files = files
        /**
         * @type {string[]}
         */
        this.pathFromRoot = pathFromRoot
    }
}


/**
 * Joins a la unix path (or url)
 * @param {string[]} path segments
 * @returns {string}
 */
function pathArrayToString(path) {
    return '/' + path.join('/')
}

/**
 * @param {ActionContext} actionContext
 * @param {Folder} currFolder 
 * @param {string[]} pathRemaining 
 * @param {string[]} pathFromRoot
 * @returns {Folder | null}
 * 
 * returns the folder from cache or fetches new
 */
async function getFromCacheOrFetch(actionContext, currFolder, pathRemaining, pathFromRoot) {

    if (currFolder == null) {
        // fetch from server
        const url = `http://localhost:8000/api/folder?url_encoded_path=${encodeURIComponent(pathArrayToString(pathFromRoot))}`
        console.log(`Folder ${pathArrayToString(pathFromRoot)} not loaded, fetching via ${url}`)
        let res
        try {
            res = await fetch(url, {
                headers: {
                    'Authorization': `Basic ${actionContext.rootState.auth.user.auth_token}`
                }
            })
        }
        catch (e) {
            console.error(e)
            return null
        }

        if (res.ok) {
            console.log('res ok')
            const folder = await res.json()
            console.log(folder)
        }
        else {
            console.error('folder req failed: ', res.status)
            throw res.statusText
        }
    }

    const next = pathRemaining.splice(0, 1)[0]
    const nchild = currFolder.childrenFolder.find(f => f.name == next)
    if (nchild == null)  {
        return null
    }
    if (path.length == 0) return nchild
    pathFromRoot.push(next)
    return getFromCacheOrFetch(nchild, pathRemaining, pathFromRoot)
}


export default {
    namespaced: true,
    state: {
        cachedFiles: null
    },
    getters: {
        /**
         * @returns {number} storage used in MB
         */
        storageUsed() {
            return 21312
        },
    },
    actions: {
        /**
         * 
         * @param {ActionContext} context 
         * @param {string[]} path 
         */
        async getFolder(context, path) {
            return await getFromCacheOrFetch(context, context.state.cachedFiles, path, [""])
        }
    }
}