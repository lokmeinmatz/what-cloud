import { UserLogin, UserLoginResponse } from '@/business/nettypes'
import { debugWindowProp } from '@/business/utils'
import { ref, watch, computed } from 'vue'
import { Node } from '../business/fs'


export enum DisplayModeType {
    Files = 'files',
    Shared = 'shared'
}

export class DisplayMode {
    mode: DisplayModeType
    sharedId?: string
    constructor(mode: DisplayModeType, shareID?: string) {
        this.mode = mode
        this.sharedId = shareID
    }

    baseUrl(): string {
        return this.mode == DisplayModeType.Files ? '/files' : `/shared/${this.sharedId}`
    }
}


// === USER STORAGE LOAD ===

const maybeUser = localStorage.getItem('user_ref')
// TODO use refresh token to revalidate auth_token
if (maybeUser) console.log('loaded user from localStorage...')

let useDarkMode

switch (localStorage.getItem('dark_mode')) {
    case 'true':
        useDarkMode = true
        break
    case 'false':
        useDarkMode = false
        break
    case null:
    default:
        // use system color theme
        if (!window.matchMedia || window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
            useDarkMode = true
        } else {
            useDarkMode = false
        }
        break;
}
console.log(`Using theme ${useDarkMode ? 'Dark':'Light'}`)
// === === === === ===

class SettingsStore {
    useDarkmode = ref(true)

    constructor() {
        // watch useDarkmode and set localStorage
        watch(this.useDarkmode, mode => {
            console.log('Updated darkmode preference')
            localStorage.setItem('dark_mode', mode.toString())
        })
    }
}

class Store {

    user = ref<UserLoginResponse | null>((maybeUser != null) ? JSON.parse(maybeUser) as UserLoginResponse : null)
    isLoggedIn = computed(() => {
        return this.user.value != null
    })

    async logIn(name: string, password: string) {
        const url = '/api/user/login'
        console.log(url)

        const body: UserLogin = {
            name,
            passwordBase64: btoa(password)
        }

        const res = await fetch(url, {
            method: 'POST',
            // eslint-disable-next-line
            body: JSON.stringify(body)
        })
        if (res.ok) {
            const resBody: UserLoginResponse = await res.json()
            console.log("response of login:", resBody)
            if (resBody.name && resBody.authToken) {
                this.user.value = resBody
                return
            }
            console.error(resBody)

        }
        else {
            const error = await res.text()
            console.log(`error: ${error}`)
            throw error
        }
    }

    // returns null if is not logged in, else fetch
    async fetchWithAuth(url: string, req?: RequestInit): Promise<Response | null> {
        
        if (this.user.value == null) return null
        req = req || {}
        if (req.headers == undefined) req.headers = {};
        (req.headers! as any)['Authorization'] = `Bearer ${this.user.value?.authToken}`
        return fetch(url, req)
    }

    displayMode = ref<DisplayMode>(new DisplayMode(DisplayModeType.Files))
    rootNode = ref<Node | null>(null)
    baseUrl = location.protocol + '//' + location.host
    settings = new SettingsStore()
}

export const store = new Store()

store.settings.useDarkmode.value = useDarkMode

debugWindowProp('store', store)

watch(store.user, user => {
    console.log('updated localStorage user')
    if (user != null) localStorage.setItem('user_ref', JSON.stringify(user))
    else localStorage.removeItem('user_ref')
})