import Vue from 'vue'
import Vuex from 'vuex'
import auth from './modules/auth'
import { File } from '../business/fs'


Vue.use(Vuex)

export default new Vuex.Store({
  strict: true,
  modules: {
    auth,
  },
  state: {
    currFileInfo: null
  },
  mutations: {
    displayFileInfo(state, file) {
      state.currFileInfo = file
    }
  }
})
