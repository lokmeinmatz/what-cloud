<template>
  <div id="app">
    <nav class="navbar navbar-expand-md navbar-dark bg-dark">
      <router-link class="navbar-brand" to="/">
        WHAT
        <span>CLOUD</span>
      </router-link>
      <button
        class="navbar-toggler"
        type="button"
        data-toggle="collapse"
        data-target="#navbarSupportedContent"
        aria-controls="navbarSupportedContent"
        aria-expanded="false"
        aria-label="Toggle navigation"
      >
        <span class="navbar-toggler-icon"></span>
      </button>
      <div class="collapse navbar-collapse" id="navbarSupportedContent">
        <ul class="navbar-nav mr-auto">
          <li class="nav-item">
            <router-link class="nav-link" to="/files">Files</router-link>
          </li>
          <li class="nav-item">
            <router-link class="nav-link" to="/shares">MyShares</router-link>
          </li>
        </ul>
        <div class="nav-item dropdown">
          <button 
            class="btn dropdown-toggle btn-dark"
            href="#"
            id="navbarDropdown"
            role="button"
            data-toggle="dropdown"
            aria-haspopup="true"
            aria-expanded="false"
          >Gear</button>
          <div class="dropdown-menu dropdown-menu-right" aria-labelledby="navbarDropdown">
            <router-link class="nav-link dropdown-item" to="/settings">Settings</router-link>
            <div class="dropdown-divider"></div>
            <a class="nav-link dropdown-item">Logout</a>
          </div>
        </div>
      </div>
    </nav>
    <router-view />
  </div>
</template>

<style scoped>
.navbar-brand > span {
  font-weight: 100;
}

</style>
