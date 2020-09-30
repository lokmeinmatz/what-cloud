import { createRouter, createWebHistory, Router, RouteRecordRaw } from 'vue-router'
import Home from '../views/Home.vue'
import Files from '../views/Files.vue'
import { DisplayMode, DisplayModeType, store } from '../store'
import * as Nprogress from 'nprogress'

const routes: Array<RouteRecordRaw> = [
  {
    path: '/',
    name: 'Home',
    component: Home
  },
  {
    path: '/files/:fpath*',
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
    path: '/shared/:sharedId/:fpath*',
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

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes
})


router.beforeEach(async (to, from, next) => {
  Nprogress.start()
  //console.log('to', to, to.params)
  // check if path is /shared or /files
  if (to.path.startsWith('/files')) {
    store.displayMode.value = new DisplayMode(DisplayModeType.Files)
  } else if (to.path.startsWith('/shared')) {
    const s = to.path.split('/').filter(s => s.length > 0)
    //console.log(s)
    if (s.length >= 2) 
    
    store.displayMode.value = new DisplayMode(DisplayModeType.Shared, s[1])
  }

  //TODO check if logged in
  if (to.path == '/login' || to.path == '/logout' || to.path.startsWith('/shared')) return next()
  if (store.auth.user.value == null) {
    console.warn('no user logged in. redirecting to /login')
    return next('/login')
  }


  const loginState = await fetch('/api/user', {
    headers: {
      'Authorization': `Bearer ${store.auth.user.value?.authToken}`
    }
  })

  if (loginState.status == 200 && store.auth.user.value != null) next()
  else {
    store.auth.user.value = null
    //console.log('Not logged in, redirecting to login')
    next('/login')

  }
})

router.afterEach(() => {
  Nprogress.done()
});
(window as unknown as {router: Router}).router = router
export default router
