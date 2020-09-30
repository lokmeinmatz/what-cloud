import { DisplayModeType, store } from '../store'
import router from '../router'
import { debugWindowProp, proxyAwareEqual } from './utils'
import { err, ok, Result } from 'neverthrow'
import { NetNode } from './nettypes'

export function pathArrayToString(path: string[]): string {
    //console.log(path)
    return '/' + path.join('/')
}

export enum GetNodeError {
    ServerNotReachable,
    NodeNotExisiting
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
    ownedBy: string

    constructor(name: string, pathFromRoot: string[], fetched: boolean, shared: string | null, ownedBy: string) {
        this.name = name
        this.pathFromRoot = pathFromRoot
        this.fetched = fetched != undefined ? fetched : false
        this.shared = shared
        this.ownedBy = ownedBy
    }

    async fetch(): Promise<Result<boolean, string>> {

        if (this.fetched) return ok(false)
        let url: string
        if (store.displayMode.value?.mode == DisplayModeType.Files) url = `/api/node?file_path=${encodeURIComponent(this.path())}`
        else if (store.displayMode.value?.sharedId != undefined) url = `/api/node?file_path=${encodeURIComponent(this.path())}&shared_id=${store.displayMode.value?.sharedId}`
        else return err('neither owned node or shared with id in storage.displayMode')
        console.log(`Node ${this.path()} not loaded, fetching via ${url}`)
        let res
        try {
            res = await fetch(url, {
                headers: {
                    'Authorization': `Bearer ${store.auth.user.value?.authToken}`
                }
            })
        }
        catch (e) {
            console.error(e)
            return err(e)
        }

        if (res.ok) {
            //console.log('res ok')
            const snode: NetNode = await res.json()
            console.log('fetched val:', snode)
            if (this.pathFromRoot.length > 0 && this.name != snode.name) {
                console.error('fetched name != local name')
                return err('Wrong Node name')
            } else if (this.pathFromRoot.length == 0) {
                console.log('updated root name to ', snode.name)
                this.name = snode.name

            }

            if (!proxyAwareEqual(this.pathFromRoot, snode.pathFromRoot)) {
                console.error('pathFromRoot differ', this.pathFromRoot, snode.pathFromRoot)
                return err('pathFromRoot differ')
            }
            if (snode.metadata.type == NodeType.Folder && snode.childrenFolder != undefined && snode.files != undefined) {
                if (this.type != NodeType.Folder && this.type != NodeType.Node) {
                    console.error('fetched Folder, but local Node is File')
                    return err('Got Folder expected File')
                }
                /* eslint-disable @typescript-eslint/no-use-before-define */
                (this as Folder).children = [
                    ...snode.childrenFolder.map((f: string) => new Folder(f, undefined, snode.pathFromRoot.concat([f]), null, snode.ownedBy)),
                    ...snode.files.map((f: string) => new File(f, snode.pathFromRoot.concat([f]).filter((e: string) => e.length > 0), null, snode.ownedBy))
                ]
                /* eslint-enable @typescript-eslint/no-use-before-define */

            } else if (snode.metadata.type == NodeType.File) {
                if (this.type != NodeType.File && this.type != NodeType.Node) {
                    console.error('fetched File, but local Node is Folder')
                    return err('Got File expected Folder')
                }
            } else {
                console.error('unknown node type: ', snode)
                return err('Unknown Node type')
            }

            this.size = snode.metadata.size
            this.lastModified = snode.metadata.lastModified
            this.fetched = true
            // cast undefined to null, is there a more elegant way???
            this.shared = snode.metadata.shared == undefined ? null : snode.metadata.shared
            this.ownedBy = snode.ownedBy

        }
        else {
            console.error('node req failed: ', res.status)
            if (res.status == 401) {
                //store.auth.user.value = null
                router.push('/logout')
                //alert('You need to log in!')
            }

            return err(res.statusText)
        }

        return ok(true)
    }

    path(): string {
        return '/' + this.pathFromRoot.join('/')
    }

    sharedLink(): string | null {
        if (this.shared == null) return null
        return `${store.baseUrl}/shared/${this.shared}/`
    }

    downloadLink(): string {
        return `/api/download/file?path=${encodeURIComponent(this.path())}&token=${store.auth.user.value?.authToken}`
    }

    async setShared(enabled: boolean) {
        const url = `/api/folder/shared?path=${encodeURIComponent(pathArrayToString(this.pathFromRoot))}${enabled ? '&enabled=true' : ''}`
        //console.log(`Updating shared setting for node ${this.path()}`)

        const res = await fetch(url, {
            method: 'PATCH',
            headers: {
                'Authorization': `Bearer ${store.auth.user.value?.authToken}`
            }
        })
        if (res.status != 200) {
            console.error('Failed to set shared: ', res)
            debugWindowProp('sharedFail', this)
            return false
        }
        const id = await res.text()


        console.log('shared update:', id)
        if (id.length == 0) this.shared = null
        else this.shared = id
    }
}


export class Folder extends Node {

    children?: Node[]

    constructor(name: string, children: Node[] | undefined, pathFromRoot: string[], shared: string | null, ownedBy: string) {
        super(name, pathFromRoot, children != null, shared, ownedBy)

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
    constructor(name: string, pathFromRoot: string[], shared: string | null, ownedBy: string) {
        super(name, pathFromRoot, false, shared, ownedBy)

        this.type = NodeType.File

    }

    ext(): string {
        const all = this.name.split(".")
        if (all.length <= 1) return ""
        return all[all.length - 1]
    }


}


async function getNodeCacheOrFetch(currNode: Node, pathRemaining: string[], pathFromRoot: string[]): Promise<Result<Node, GetNodeError>> {

    if (!currNode.fetched) {
        // fetch from server
        await currNode.fetch()
        if (currNode.pathFromRoot.length == 0)
            store.rootNode.value = currNode
    }
    if (pathRemaining.length == 0) return ok(currNode)
    const next = pathRemaining.splice(0, 1)[0]
    // we know chrrNode is a folder
    const nchild = (currNode as Folder).children?.find(f => f.name == next)

    //debugger
    if (nchild == undefined) {
        console.error(currNode, next)
        return err(GetNodeError.NodeNotExisiting)
    }
    //console.log('next', next)
    pathFromRoot.push(next)
    return getNodeCacheOrFetch(nchild, pathRemaining, pathFromRoot)
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
export async function getNode(path: string | string[]): Promise<Result<Node, GetNodeError>> {
    if (typeof path == 'string') path = path.split('/').filter(e => e.length > 0)
    /**
     * @type {Node}
     */
    const curr = await getNodeCacheOrFetch(store.rootNode.value as Node, path, [])

    //console.log('getNode', curr)
    if (curr.isOk() && !curr.value.fetched) {
        console.error('node isnt fetched, something went wrong')
        err(GetNodeError.ServerNotReachable)
    }

    return curr
}

export async function updateShared(shared: Array<{ path: string; share_id: string }>): Promise<Node[]> {
    const res = []
    for (const entry of shared) {
        const f = await getNode(entry.path)
        if (f.isOk()) {
            res.push(f.value)
            f.value.shared = entry.share_id
        }
    }
    return res
}


export function reset() {
    console.log('set root node unfetched')
    store.rootNode.value = new Folder('', undefined, [], null, store.auth.user.value?.userId || "unknown")
}

reset()


