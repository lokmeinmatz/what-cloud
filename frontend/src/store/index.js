import Vue from 'vue'
import Vuex from 'vuex'
import auth from './modules/auth'
import { Node } from '../business/fs'


Vue.use(Vuex)

export default new Vuex.Store({
  strict: true,
  modules: {
    auth,
  }
})
