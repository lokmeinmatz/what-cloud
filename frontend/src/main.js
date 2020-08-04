import Vue from 'vue'
import App from './App.vue'
import './registerServiceWorker'
import router from './router'
import store from './store'

Vue.config.productionTip = false

new Vue({
  router,
  store,
  render: h => h(App)
}).$mount('#app')

window.debugLogin = () => {
  console.warn('DEBUG LOGIN USED\nRemove before production push')
  store.commit('auth/setUser', {name: 'Debug', profile_picture_url: null, auth_token: '0123456789abcdef'})
  router.push('/')
}