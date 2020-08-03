import store from '../store'
import router from '../router'

export class File {
    /**
     * @param {Object} obj
     * @param {string} obj.name
     * @param {string[]} obj.childrenFolder
     * @param {string[]} obj.files
     * @param {string[]} obj.pathFromRoot
     */
    constructor({name, childrenFolder, files, pathFromRoot}) {
        this.name = name
        
        this.pathFromRoot = pathFromRoot
        if (childrenFolder == undefined || files == undefined ||pathFromRoot == undefined) {
            this.fetched = false
            return
        }
        else {
            this.fetched = true
        }

        /**
         * @type {File[]}
         */
        this.childrenFolder = childrenFolder.map(f => new File({name: f}))
        this.files = files.map(f => new File({name: f}))
        /**
         * @type {string[]}
         */
    }

    allContents() {
        const res = []
        res.push(...this.childrenFolder.map(f => { return {type: 'folder', name: f.name}}))
        res.push(...this.files.map(f => { 
            const exts = f.name.split('.')
            let r = {type: 'file', name: f.name}

            if (exts.length > 1) {
                r.ext = exts[exts.length - 1]
            }
            return r
        }))
        return res
    }

    path() {
        return '/' + this.pathFromRoot.join('/')
    }
}

let rootFile = new File({name: "", pathFromRoot: []})
console.log('set root file unfetched')
window.rootFile = rootFile
/**
 * 
 * @param {string[]} path 
 */
export async function getFolder(path) {
    console.log('getFolder', path)
    const res = await getFromCacheOrFetch({currFolder: rootFile, pathRemaining: path, pathFromRoot: [], parentFolder: null})
    console.log(res)
    return res
}


/**
 * @param {Object} obj
 * @param {File} obj.currFolder 
 * @param {string[]} obj.pathRemaining 
 * @param {string[]} obj.pathFromRoot
 * @param {File} obj.parentFolder
 * @returns {File | null}
 * 
 * returns the folder from cache or fetches new
 */
async function getFromCacheOrFetch({currFolder, pathRemaining, pathFromRoot, parentFolder}) {

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
            console.log('res ok')
            const folder = await res.json()
            currFolder = new File(folder)
            if (currFolder.pathFromRoot.length == 0) {
                rootFile = currFolder
            } else {
                parentFolder.childrenFolder = parentFolder.childrenFolder.filter(f => {
                    return f.name != currFolder.name
                })
                parentFolder.childrenFolder.push(currFolder)
                //console.log('parent', parentFolder)
            }
            //console.log(currFolder)
        }
        else {
            console.error('folder req failed: ', res.status)
            if (res.status == 401) {
                store.commit('auth/setUser', null, {root: true})
                router.push('/login')
                alert('You need to log in!')
            }

            throw res.statusText
        }
    }
    if (pathRemaining.length == 0) return currFolder
    const next = pathRemaining.splice(0, 1)[0]
    const nchild = currFolder.childrenFolder.find(f => f.name == next)
    //debugger
    if (nchild == null)  {
        return null
    }
    console.log('next', next)
    pathFromRoot.push(next)
    return getFromCacheOrFetch({currFolder: nchild, pathRemaining, pathFromRoot, parentFolder: currFolder})
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