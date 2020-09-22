import { ref, watch } from 'vue'
import { Node } from '../business/fs'

export interface User {
    auth_token: string;
    name: string;
}

export class DisplayMode {
    mode: string
    constructor(mode: string) {
        this.mode = mode
    }
}
export class MyFilesDisplayMode extends DisplayMode {
    constructor() {
        super('files')
    }
}

export class SharedDisplayMode extends DisplayMode {
    sharedId: string
    constructor(sharedId: string) {
        super('shared')
        this.sharedId = sharedId
    }
}

const maybeUser = localStorage.getItem('user_ref')
// TODO use refresh token to revalidate auth_token
if (maybeUser) console.log('loaded user from localStorage...')

export const store =  {
    auth: {
        user: ref<User | null>((maybeUser != null) ? JSON.parse(maybeUser) : null),
        async logIn(name: string, password: string) {
            const url = '/api/user/login'
            console.log(url)
            const res = await fetch(url, {
                method: 'POST',
                // eslint-disable-next-line
                body: JSON.stringify({name, password_base64: btoa(password)})
            })
            if (res.ok) {
                const resBody = await res.json()
                console.log("response of login:", resBody)
                if (resBody.name && resBody.auth_token) {
                    store.auth.user.value = resBody
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
    },
    displayMode: ref<DisplayMode>(new MyFilesDisplayMode()),
    rootNode: ref<Node | null>(null),
    baseUrl: location.protocol + '//' +location.host
} as const
console.log(store.auth.user.value)
watch(store.auth.user, user => {
    console.log('updated localStorage user')
    if (user != null) localStorage.setItem('user_ref', JSON.stringify(user))
    else localStorage.removeItem('user_ref')
})