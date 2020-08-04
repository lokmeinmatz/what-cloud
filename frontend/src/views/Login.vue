<template>
  <main class="container-md">
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
          <input name="password" autocomplete="password" type="password" v-model="password_raw" class="form-control" id="password" placeholder="Password">
        </div>
        <button class="btn btn-success" type="submit">Login</button>
      </form>
    </div>
  </main>
</template>
<script>


export default {
  name: 'Login',
  data() {
    return {
      name: '',
      password_raw: '',
      logging_in: false
    }
  },
  methods: {
    async login() {
      if (this.name.length < 1 || this.password_raw.length < 1) {
        alert('Please enter username and password')
        return
      }
      console.log('logging in...')
      this.logging_in = true
      try {
        await this.$store.dispatch('auth/login', {name: this.name, password: this.password_raw})
        this.logging_in = false
        this.$router.push('/')
      } catch (error) {
        alert(error)
        this.logging_in = false
      }

    }
  }
}
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
