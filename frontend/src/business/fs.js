import store from '../store'
import router from '../router'
import { state } from './globalState'

async function delay(ms) {
    return new Promise((res, rej) => {
        setTimeout(() => res(), ms)
    })
} 

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
        this.fetched = fetched != undefined ? fetched : false
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
}

/**
 * 
 * @param {string | string[]} path either / separated string or allready split
 * @returns {Node}
 */
export async function getNode(path) {
    if (typeof path == 'string') path = path.split('/').filter(e => e.length > 0)
    let pIdx = 0
    /**
     * @type {Folder}
     */
    let curr = window.rootNode
    if (curr == undefined) {
        console.error('rootNode undefined, tried to getNode()')
        return undefined
    }
    while (path.length > pIdx && curr != undefined) {
        let seg = path[pIdx]
        
        if (!curr.fetched) curr = await getNodeCacheOrFetch({ currNode: window.rootNode, pathRemaining: path, pathFromRoot: [], parentFolder: null })
        
        //debugger
        // search for child that matches seg
        curr = curr.children.find(n => n.name == seg)
        pIdx++
    }
    return curr
}

/**
 * 
 * @param {Array<{path: string, share_id: string}>} shared 
 * @returns {Node[]} all files / folders that are shared
 */
export async function updateShared(shared) {
    let res = []
    for(let entry of shared) {
        const f = await getNode(entry.path)
        if (f) {
            res.push(f)
            f.shared = entry.share_id
        }
    }
    return res
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
 * @param {Object} obj
 * @param {Node} obj.currNode 
 * @param {string[]} obj.pathRemaining 
 * @param {string[]} obj.pathFromRoot
 * @param {Folder} obj.parentFolder
 * @returns {File | Folder | null}
 * 
 * returns the folder from cache or fetches new
 */
async function getNodeCacheOrFetch({ currNode, pathRemaining, pathFromRoot, parentFolder }) {

    if (!currNode.fetched) {
        // fetch from server
        const url = `/api/node?url_encoded_path=${encodeURIComponent(pathArrayToString(pathFromRoot))}`
        console.log(`Node ${pathArrayToString(pathFromRoot)} not loaded, fetching via ${url}`)
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
            const snode = await res.json()
            console.log('fetched val:', snode)
            if (snode.type == 'folder') {
                console.log('got folder')
                currNode = new Folder({
                    children: [
                        ...snode.childrenFolder.map(f => new Folder({ name: f, pathFromRoot: snode.pathFromRoot.concat([f]), shared: null })),
                        ...snode.files.map(f => new File({ name: f, pathFromRoot: snode.pathFromRoot.concat([f]).filter(e => e.length > 0), shared: null }))],
                        name: snode.name,
                        pathFromRoot: snode.pathFromRoot
                    })
                } else if (snode.type == 'file') {
                    console.log('got file')
                    currNode = new File({
                    name: snode.name,
                    pathFromRoot: snode.pathFromRoot,
                })
            } else {
                console.error('unknown node type: ', snode.type)
                return null
            }

            currNode.size = res.size
            currNode.lastModified = res.lastModified
            //this.fetched = true
            currNode.shared = res.shared

            if (currNode.pathFromRoot.length == 0) {
                window.rootNode = currNode
                console.log('new root node:', window.rootNode)
            } else {
                parentFolder.children = parentFolder.children.filter(f => {
                    // folders and files cannot have the same name
                    return !(f.name == currNode.name)
                })
                parentFolder.children.push(currNode)
                //console.log('parent', parentFolder)
            }
        }
        else {
            console.error('node req failed: ', res.status)
            if (res.status == 401) {
                store.commit('auth/setUser', null, { root: true })
                router.push('/login')
                alert('You need to log in!')
            }

            throw res.statusText
        }
    }
    if (pathRemaining.length == 0) return currNode
    const next = pathRemaining.splice(0, 1)[0]
    const nchild = currNode.children.find(f => f.name == next)
    //debugger
    if (nchild == null) {
        return null
    }
    //console.log('next', next)
    pathFromRoot.push(next)
    return getNodeCacheOrFetch({ currNode: nchild, pathRemaining, pathFromRoot, parentFolder: currNode })
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
