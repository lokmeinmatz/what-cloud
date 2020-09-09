import Vue from 'vue'
import VueRouter from 'vue-router'
import Home from '../views/Home.vue'
import Files from '../views/Files.vue'
import store from '../store'
import { state } from '../business/globalState'

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
    path: '/shared',
    name: 'ShareList',
    // route level code-splitting
    // this generates a separate chunk (about.[hash].js) for this route
    // which is lazy-loaded when the route is visited.
    component: () => import(/* webpackChunkName: "about" */ '../views/SharedList.vue')
    //component: SharedList
  },
  {
    path: '/shared*',
    name: 'Share',
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
  
  // check if path is /shared or /files
  if (to.path.startsWith('/files')) {
    state.fileDisplayState.emit({mode: 'files'})
  } else if (to.path.startsWith('/shared')) {
    let s = to.path.split('/').filter(s => s.length > 0)
    //console.log(s)
    if (s.length >= 2) 
    state.fileDisplayState.emit({mode: 'shared', shareID: s[1]})
  }

  //TODO check if logged in
  if (to.path == '/login' || to.path.startsWith('/shared')) return next()
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
