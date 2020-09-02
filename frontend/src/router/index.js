import Vue from 'vue'
import VueRouter from 'vue-router'
import Home from '../views/Home.vue'
import Files from '../views/Files.vue'
import store from '../store'

import * as Nprogress from 'nprogress'

Vue.use(VueRouter)

const routes = [
  {
    path: '/',
    name: 'Home',
    component: Home
  },
  {
    path: '/files*',
    name: 'Files',
    // route level code-splitting
    // this generates a separate chunk (about.[hash].js) for this route
    // which is lazy-loaded when the route is visited.
    //component: () => import(/* webpackChunkName: "about" */ '../views/Files.vue')
    component: Files
  },
  {
    path: '/login',
    name: 'Login',
    // route level code-splitting
    // this generates a separate chunk (about.[hash].js) for this route
    // which is lazy-loaded when the route is visited.
    //component: () => import(/* webpackChunkName: "about" */ '../views/Files.vue')
    component: () => import(/* webpackChunkName: "login" */ '../views/Login.vue')
  },
  {
    path: '/logout',
    name: 'Logout',
    component: () => import(/* webpackChunkName: "logout" */ '../views/Logout.vue')
  }
]

const router = new VueRouter({
  mode: 'history',
  base: process.env.BASE_URL,
  routes
})

router.beforeEach(async (to, from, next) => {
  Nprogress.start()
  //TODO check if logged in
  if (to.path == '/login') return next()
  if (store.state.auth.user == null) return next('/login')
  const loginState = await fetch('/api/user', {
    headers: {
      'Authorization': `Bearer ${store.state.auth.user.auth_token}`
    }
  })

  if (loginState.status == 200 && store.getters['auth/isLoggedIn']) next()
  else {
    //console.log('Not logged in, redirecting to login')
    next('/login')

  }
})

router.afterEach(() => {
  Nprogress.done()
})

export default router
