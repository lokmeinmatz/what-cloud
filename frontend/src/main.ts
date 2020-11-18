import { App, createApp } from 'vue'
import MyApp from './App.vue'
import router from './router'


const app = createApp(MyApp)

app.use(router)
app.mount('#app-wrapper')
