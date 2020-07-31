const maybeUser = localStorage.getItem('user_ref')
// TODO use refresh token to revalidate auth_token
if (maybeUser) console.log('loaded user from localStorage...')

export default {
    namespaced: true,
    state: {
        user: maybeUser ? JSON.parse(maybeUser) : null
    },
    getters: {
        isLoggedIn: state => state.user != null
    },
    mutations: {
        setUser(state, user) {
            if (user != null) localStorage.setItem('user_ref', JSON.stringify(user))
            else localStorage.removeItem('user_ref')
            state.user = user
        }
    },
    actions: {
        async login(context, {name, password}) {
            const url = 'http://localhost:8000/api/user/login'
            console.log(url)
            const res = await fetch(url, {
                method: 'POST',
                body: JSON.stringify({name, password_base64: btoa(password)})
            })
            if (res.ok) {
                const res_body = await res.json()
                console.log("response of login:", res_body)
                if (res_body.name && res_body.auth_token) {
                    context.commit('setUser', res_body)
                    return
                }
                console.error(res_body)

            }
            else {
                const error = await res.text()
                console.log(`error: ${error}`)
                throw error
            }
        }
    }
}