import store from '../store'
import router from '../router'
import { state } from './globalState'

export class Node {
    /**
     * @param {Object} obj
     * @param {string} obj.name
     * @param {string[]} obj.pathFromRoot
     * @param {boolean} obj.fetched
     * @param {string | null} obj.shared
     */
    constructor({ name, pathFromRoot, fetched, shared }) {
        this.name = name
        this.pathFromRoot = pathFromRoot
        this.fetched = fetched != null ? fetched : false
        this.type = 'node'
        this.size = -1
        this.lastModified = '',
        this.shared = shared
    }

    path() {
        return '/' + this.pathFromRoot.join('/')
    }

    sharedLink() {
        if (this.shared == null) return null
        return `${state.baseUrl}/shared/${this.shared}/`
    }

    downloadLink() {
        return `/api/download/file?path=${encodeURIComponent(this.path())}&token=${store.state.auth.user.auth_token}`
    }

    /**
     * 
     * @param {boolean} shared 
     */
    async setShared(enabled) {
        const url = `/api/folder/shared?url_encoded_path=${encodeURIComponent(pathArrayToString(this.pathFromRoot))}${enabled ? '&enabled=true' : ''}`
        console.log(`Updating shared setting for node ${this.path()}`)

        let res
        try {
            res = await fetch(url, {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${store.state.auth.user.auth_token}`
                }
            })
            if (res.status != 200) {
                console.error(res)
                return false
            }
            res = await res.text()
        }
        catch (e) {
            console.error(e)
            return false
        }

        console.log('shared update:', res)
        if (res.length == 0) this.shared = null
        else this.shared = res 
    }

    async loadMetadata() {
        const url = `/api/metadata?url_encoded_path=${encodeURIComponent(pathArrayToString(this.pathFromRoot))}`
        console.log(`Fetching metadata via ${url}`)
        let res
        try {
            res = await fetch(url, {
                headers: {
                    'Authorization': `Bearer ${store.state.auth.user.auth_token}`
                }
            })
            if (res.status != 200) {
                return false
            }
            res = await res.json()
        }
        catch (e) {
            console.error(e)
            return false
        }

        //console.log(res)
        this.size = res.size
        this.lastModified = res.lastModified
        this.fetched = true
        this.shared = res.shared
    }
}


export class Folder extends Node {
    /**
     * @param {Object} obj
     * @param {string} obj.name
     * @param {Node[]} obj.children
     * @param {string[]} obj.pathFromRoot
     * @param {string | null} obj.shared
     */
    constructor({ name, children, pathFromRoot, shared }) {
        super({ name, pathFromRoot, fetched: children != null, shared })

        this.type = 'folder'
        /**
         * @type {Node[]}
         */
        this.children = children
    }
}

export class File extends Node {
    /**
     * Maybe move fetched to Folder so file has no fetched, or indicate metadata fetch?
     * @param {Object} obj
     * @param {string} obj.name
     * @param {string[]} obj.pathFromRoot
     * @param {string | null} obj.shared
     */
    constructor({ name, pathFromRoot, shared }) {
        super({ name, pathFromRoot, shared })

        this.type = 'file'

    }

    /**
     * @returns {string} extension of file
     */
    ext() {
        const all = this.name.split(".")
        if (all.length <= 1) return ""
        return all[all.length - 1]
    }


}

export function reset() {
    console.log('set root node unfetched')
    window.rootNode = new Node({ name: "", pathFromRoot: [] })
}

reset()

/**
 * 
 * @param {string[]} path 
 */
export async function getFolder(path) {
    console.log('getFolder', path)
    const res = await getFolderCacheOrFetch({ currFolder: window.rootNode, pathRemaining: path, pathFromRoot: [], parentFolder: null })
    //console.log(res)
    return res
}


/**
 * @param {Object} obj
 * @param {Folder} obj.currFolder 
 * @param {string[]} obj.pathRemaining 
 * @param {string[]} obj.pathFromRoot
 * @param {Folder} obj.parentFolder
 * @returns {Folder | null}
 * 
 * returns the folder from cache or fetches new
 */
async function getFolderCacheOrFetch({ currFolder, pathRemaining, pathFromRoot, parentFolder }) {

    if (!currFolder.fetched) {
        // fetch from server
        const url = `/api/folder?url_encoded_path=${encodeURIComponent(pathArrayToString(pathFromRoot))}`
        console.log(`Folder ${pathArrayToString(pathFromRoot)} not loaded, fetching via ${url}`)
        let res
        try {
            res = await fetch(url, {
                headers: {
                    'Authorization': `Bearer ${store.state.auth.user.auth_token}`
                }
            })
        }
        catch (e) {
            console.error(e)
            return null
        }

        if (res.ok) {
            //console.log('res ok')
            const folder = await res.json()
            console.log('fetched val:', folder)
            currFolder = new Folder({
                children: [
                    ...folder.childrenFolder.map(f => new Folder({ name: f, pathFromRoot: folder.pathFromRoot.concat([f]), shared: null })),
                    ...folder.files.map(f => new File({ name: f, pathFromRoot: folder.pathFromRoot.concat([f]).filter(e => e.length > 0), shared: null }))],
                name: folder.name,
                pathFromRoot: folder.pathFromRoot
            })
            //console.log(currFolder.children)
            if (currFolder.pathFromRoot.length == 0) {
                window.rootNode = currFolder
                console.log('new root node:', window.rootNode)
            } else {
                parentFolder.children = parentFolder.children.filter(f => {
                    return !(f.name == currFolder.name && f instanceof Folder)
                })
                parentFolder.children.push(currFolder)
                //console.log('parent', parentFolder)
            }
            //console.log(currFolder)
        }
        else {
            console.error('folder req failed: ', res.status)
            if (res.status == 401) {
                store.commit('auth/setUser', null, { root: true })
                router.push('/login')
                alert('You need to log in!')
            }

            throw res.statusText
        }
    }
    if (pathRemaining.length == 0) return currFolder
    const next = pathRemaining.splice(0, 1)[0]
    const nchild = currFolder.children.find(f => f.name == next && f instanceof Folder)
    //debugger
    if (nchild == null) {
        return null
    }
    //console.log('next', next)
    pathFromRoot.push(next)
    return getFolderCacheOrFetch({ currFolder: nchild, pathRemaining, pathFromRoot, parentFolder: currFolder })
}


/**
 * Joins a la unix path (or url)
 * @param {string[]} path segments
 * @returns {string}
 */
export function pathArrayToString(path) {
    //console.log(path)
    return '/' + path.join('/')
}

/**
 * @param {Object} obj
 * @param {ActionContext} obj.actionContext
 * @param {File} obj.currFolder
 * @param {string[]} obj.pathRemaining
 * @param {string[]} obj.pathFromRoot
 * @param {File} obj.parentFolder
 * @returns {File | null}
 *
 * returns the folder from cache or fetches new
 */