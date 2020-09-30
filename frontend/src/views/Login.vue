<template>
  <main class="container">
    <div class="card">
      <div class="card-img-top">
      </div>
      <form class="card-body" @submit.prevent="login">
        <h4 class="card-title">User-Login</h4>
        <div class="form-group">
          <label for="username">Username</label>
          <input name="username" autocomplete="username" autofocus type="string" v-model="name" class="form-control" id="username" placeholder="Enter Username">
        </div>
        <div class="form-group">
          <label for="password">Password</label>
          <input name="password" autocomplete="password" type="password" v-model="passwordRaw" class="form-control" id="password" placeholder="Password">
        </div>
        <button class="btn btn-success" type="submit">Login</button>
      </form>
    </div>
  </main>
</template>
<script lang="ts">
import { defineComponent } from 'vue'
import { store } from '../store'

export default defineComponent({
  name: 'Login',
  data() {
    return {
      name: '',
      passwordRaw: '',
      loggingIn: false
    }
  },
  methods: {
    async login() {
      if (this.name.length < 1 || this.passwordRaw.length < 1) {
        alert('Please enter username and password')
        return
      }
      console.log('logging in...')
      this.loggingIn = true
      try {
        await store.logIn(this.name, this.passwordRaw)
        this.loggingIn = false
        this.$router.push('/')
      } catch (error) {
        alert(error)
        this.loggingIn = false
      }

    }
  }
})
</script>
<style scoped>
h1 {
  text-align: center;
  margin-top: 1em;
}

.card {
  margin-top: 1rem;
  width: 100%;
}

</style>
