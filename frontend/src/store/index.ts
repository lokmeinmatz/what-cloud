import { UserLogin, UserLoginResponse } from '@/business/nettypes'
import { ref, watch } from 'vue'
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

const maybeUser = localStorage.getItem('user_ref')
// TODO use refresh token to revalidate auth_token
if (maybeUser) console.log('loaded user from localStorage...')

export const store =  {
    auth: {
        user: ref<UserLoginResponse | null>((maybeUser != null) ? JSON.parse(maybeUser) as UserLoginResponse : null),
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
    displayMode: ref<DisplayMode>(new DisplayMode(DisplayModeType.Files)),
    rootNode: ref<Node | null>(null),
    baseUrl: location.protocol + '//' +location.host
} as const
console.log(store.auth.user.value)
watch(store.auth.user, user => {
    console.log('updated localStorage user')
    if (user != null) localStorage.setItem('user_ref', JSON.stringify(user))
    else localStorage.removeItem('user_ref')
})