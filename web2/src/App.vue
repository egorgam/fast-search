<template>
  <div id="app">
    <div>
      <input type="text" v-model="query">
    </div>
    <div v-for="(result, idx) in results" :key="idx">
        <i class="text">{{result}}</i>
    </div>
  </div>
</template>

<script>
const axios = require('axios');
export default {
  name: 'app',
  data () {
    return {
      query: '',
      results: []
    }
  },
   watch: {
    query: function () {
      axios.get('http://localhost:8000/search', {
            params: {
              query: this.query
            }
          })
          .then(response => this.results = response.data["result"]);
    }
  }
}
</script>

<style>
body {
    background-color: #1f1f1f;
   }
.text {
  color: #dddddd;
}
</style>