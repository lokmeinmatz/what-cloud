import { store } from '../store'
import router from '../router'

export function pathArrayToString(path: string[]): string {
    //console.log(path)
    return '/' + path.join('/')
}


export enum NodeType {
    Node = 'node',
    File = 'file',
    Folder = 'folder'
}

export class Node {
    name: string
    pathFromRoot: string[]
    fetched: boolean
    shared: string | null
    type = NodeType.Node
    size = -1
    lastModified = ''

    constructor(name: string, pathFromRoot: string[], fetched: boolean, shared: string | null) {
        this.name = name
        this.pathFromRoot = pathFromRoot
        this.fetched = fetched != undefined ? fetched : false
        this.shared = shared
    }

    path(): string {
        return '/' + this.pathFromRoot.join('/')
    }

    sharedLink(): string | null {
        if (this.shared == null) return null
        return `${store.baseUrl}/shared/${this.shared}/`
    }

    downloadLink(): string {
        return `/api/download/file?path=${encodeURIComponent(this.path())}&token=${store.auth.user.value?.auth_token}`
    }

    async setShared(enabled: boolean) {
        const url = `/api/folder/shared?url_encoded_path=${encodeURIComponent(pathArrayToString(this.pathFromRoot))}${enabled ? '&enabled=true' : ''}`
        //console.log(`Updating shared setting for node ${this.path()}`)

        let res
        try {
            res = await fetch(url, {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${store.auth.user.value?.auth_token}`
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


export class Folder extends Node {
    
    children?: Node[]

    constructor(name: string, children: Node[] | undefined, pathFromRoot: string[], shared: string | null) {
        super(name, pathFromRoot, children != null, shared)

        this.type = NodeType.Folder

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
    constructor(name: string, pathFromRoot: string[], shared: string | null) {
        super(name, pathFromRoot, false ,shared)

        this.type = NodeType.File

    }

    ext(): string {
        const all = this.name.split(".")
        if (all.length <= 1) return ""
        return all[all.length - 1]
    }


}


async function getNodeCacheOrFetch(currNode: Node, pathRemaining: string[], pathFromRoot: string[], parentFolder: Folder | null): Promise<File | Folder> {

    if (!currNode.fetched) {
        // fetch from server
        const url = `/api/node?url_encoded_path=${encodeURIComponent(pathArrayToString(pathFromRoot))}`
        console.log(`Node ${pathArrayToString(pathFromRoot)} not loaded, fetching via ${url}`)
        let res
        try {
            res = await fetch(url, {
                headers: {
                    'Authorization': `Bearer ${store.auth.user.value?.auth_token}`
                }
            })
        }
        catch (e) {
            console.error(e)
            throw e
        }

        if (res.ok) {
            //console.log('res ok')
            const snode = await res.json()
            console.log('fetched val:', snode)
            if (snode.type == NodeType.Folder) {
                currNode = new Folder(
                    snode.name, 
                    [
                        ...snode.childrenFolder.map((f: string) => new Folder(f, undefined, snode.pathFromRoot.concat([f]),  null )),
                        ...snode.files.map((f: string) => new File(f, snode.pathFromRoot.concat([f]).filter((e: string) => e.length > 0), null ))
                    ], 
                    snode.pathFromRoot,
                    null)
                } else if (snode.type == NodeType.File) {
                    
                    currNode = new File(
                        snode.name,
                        snode.pathFromRoot,
                        null)
            } else {
                console.error('unknown node type: ', snode.type)
                throw null
            }

            currNode.size = snode.metadata.size
            currNode.lastModified = snode.metadata.lastModified
            currNode.fetched = true
            currNode.shared = snode.metadata.shared

            if (currNode.pathFromRoot.length == 0) {
                store.rootNode.value = currNode
            } else if (parentFolder != null) {
                // update node this is the child of
                parentFolder.children = parentFolder.children?.filter(f => {
                    // folders and files cannot have the same name
                    return !(f.name == currNode.name)
                })
                if (parentFolder.children == undefined) parentFolder.children = []
                parentFolder.children.push(currNode)
            }
        }
        else {
            console.error('node req failed: ', res.status)
            if (res.status == 401) {
                store.auth.user.value = null
                router.push('/login')
                alert('You need to log in!')
            }

            throw res.statusText
        }
    }
    if (pathRemaining.length == 0) return currNode
    const next = pathRemaining.splice(0, 1)[0]
    // we know chrrNode is a folder
    const nchild = (currNode as Folder).children?.find(f => f.name == next)

    //debugger
    if (nchild == undefined) {
        throw 'next child not in list of children'
    }
    //console.log('next', next)
    pathFromRoot.push(next)
    return getNodeCacheOrFetch(nchild, pathRemaining, pathFromRoot, currNode )
}





// TODO reimplement inside node?

/*
// set up listener on state nodeinfo change to fetch if isn't fetched
state.nodeInfoDisplay.subscribeWithId('fs-fetch', async fr => {
    console.log(fr)
    if (fr != null && !fr.fetched) state.nodeInfoDisplay.emit( await getNode(fr.pathFromRoot) )
})
*/


/**
 * 
 * @param {string | string[]} path either / separated string or allready split
 * @returns {Node}
 */
export async function getNode(path: string | string[]): Promise<Node> {
    if (typeof path == 'string') path = path.split('/').filter(e => e.length > 0)
    /**
     * @type {Node}
     */
    const curr = await getNodeCacheOrFetch(store.rootNode.value as Node, path, [], null )
   
    console.log('getNode', curr)
    if (!curr.fetched) {
        console.error('node isnt fetched, something went wrong')
        throw null
    }
    return curr
}

export async function updateShared(shared: Array<{path: string; share_id: string}>): Promise<Node[]> {
    const res = []
    for(const entry of shared) {
        const f = await getNode(entry.path)
        if (f) {
            res.push(f)
            f.shared = entry.share_id
        }
    }
    return res
}


export function reset() {
    console.log('set root node unfetched')
    store.rootNode.value = new Node('', [], false, null)
}

reset()


