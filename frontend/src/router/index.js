import Vue from 'vue'
import VueRouter from 'vue-router'
import Home from '../views/Home.vue'
import Files from '../views/Files.vue'

Vue.use(VueRouter)

  const routes = [
  {
    path: '/',
    name: 'Home',
    component: Home
  },
  {
    path: '/files',
    alias: '/files/*',
    name: 'Files',
    // route level code-splitting
    // this generates a separate chunk (about.[hash].js) for this route
    // which is lazy-loaded when the route is visited.
    //component: () => import(/* webpackChunkName: "about" */ '../views/Files.vue')
    component: Files
  }
]

const router = new VueRouter({
  mode: 'history',
  base: process.env.BASE_URL,
  routes
})

export default router
